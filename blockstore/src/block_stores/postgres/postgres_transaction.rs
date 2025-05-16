use std::str::FromStr;

use futures_util::pin_mut;
use log::debug;
use solana_lite_rpc_core::encoding::BinaryEncoding;
use solana_lite_rpc_core::solana_utils::hash_from_str;
use solana_lite_rpc_core::structures::epoch::EpochRef;
use solana_lite_rpc_core::{encoding::BASE64, structures::produced_block::TransactionInfo};
use solana_sdk::signature::Signature;
use solana_sdk::slot_history::Slot;
use solana_sdk::transaction::TransactionError;
use tokio::time::Instant;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::Type;
use tokio_postgres::CopyInSink;

use super::postgres_epoch::*;
use super::postgres_session::*;

#[derive(Debug)]
pub struct PostgresTransaction {
    pub signature: String,
    // TODO clarify
    pub slot: i64,
    pub err: Option<String>,
    pub cu_requested: Option<i64>,
    pub prioritization_fees: Option<i64>,
    pub cu_consumed: Option<i64>,
    pub recent_blockhash: String,
    pub message: String,
}

impl PostgresTransaction {
    pub fn new(value: &TransactionInfo, slot: Slot) -> Self {
        Self {
            signature: value.signature.to_string(),
            err: value
                .err
                .clone()
                .map(|x| BASE64.serialize(&x).ok())
                .unwrap_or(None),
            cu_requested: value.cu_requested.map(|x| x as i64),
            prioritization_fees: value.prioritization_fees.map(|x| x as i64),
            cu_consumed: value.cu_consumed.map(|x| x as i64),
            recent_blockhash: value.recent_blockhash.to_string(),
            message: BinaryEncoding::Base64.encode(value.message.serialize()),
            slot: slot as i64,
        }
    }

    pub fn to_transaction_info(&self) -> TransactionInfo {
        TransactionInfo {
            signature: Signature::from_str(self.signature.as_str()).unwrap(),
            err: self
                .err
                .as_ref()
                .and_then(|x| BASE64.deserialize::<TransactionError>(x).ok()),
            cu_requested: self.cu_requested.map(|x| x as u32),
            prioritization_fees: self.prioritization_fees.map(|x| x as u64),
            cu_consumed: self.cu_consumed.map(|x| x as u64),
            recent_blockhash: hash_from_str(&self.recent_blockhash).expect("valid blockhash"),
            message: BinaryEncoding::Base64
                .deserialize(&self.message)
                .expect("serialized message"),
            // TODO readable_accounts etc.
            readable_accounts: vec![],
            writable_accounts: vec![],
            is_vote: false,
            address_lookup_tables: vec![],
        }
    }

    pub fn build_create_table_statement(epoch: EpochRef) -> String {
        let schema = PostgresEpoch::build_schema_name(epoch);
        format!(
            r#"
                -- lookup table; maps signatures to generated int8 transaction ids
                -- no updates or deletes, only INSERTs
                CREATE TABLE {schema}.transaction_ids(
                    transaction_id bigserial PRIMARY KEY WITH (FILLFACTOR=90),
                    -- never put sig on TOAST
                    signature text STORAGE PLAIN NOT NULL,
                    UNIQUE(signature)
                ) WITH (FILLFACTOR=100);

                -- parameter 'schema' is something like 'rpc2a_epoch_592'
                CREATE TABLE IF NOT EXISTS {schema}.transaction_blockdata(
                    -- transaction_id must exist in the transaction_ids table
                    transaction_id bigint PRIMARY KEY WITH (FILLFACTOR=90),
                    slot bigint NOT NULL,
                    cu_requested bigint,
                    prioritization_fees bigint,
                    cu_consumed bigint,
                    recent_blockhash text NOT NULL,
                    err text,
                    message text NOT NULL
                    -- model_transaction_blockdata
                ) WITH (FILLFACTOR=90,TOAST_TUPLE_TARGET=128);
                CREATE INDEX idx_slot ON {schema}.transaction_blockdata USING btree (slot) WITH (FILLFACTOR=90);
            "#,
            schema = schema
        )
    }

    // removed the foreign key as it slows down inserts
    pub fn build_foreign_key_statement(epoch: EpochRef) -> String {
        let schema = PostgresEpoch::build_schema_name(epoch);
        format!(
            r#"
                ALTER TABLE {schema}.transaction_blockdata
                ADD CONSTRAINT fk_transactions FOREIGN KEY (slot) REFERENCES {schema}.blocks (slot);
            "#,
            schema = schema
        )
    }

    pub async fn save_transactions_from_block(
        postgres_session: PostgresSession,
        epoch: EpochRef,
        transactions: &[Self],
    ) -> anyhow::Result<()> {
        let schema = PostgresEpoch::build_schema_name(epoch);

        let statmement = r#"
            CREATE TEMP TABLE IF NOT EXISTS transaction_raw_blockdata(
                signature text,
                slot bigint,
                cu_requested bigint,
                prioritization_fees bigint,
                cu_consumed bigint,
                recent_blockhash text STORAGE PLAIN,
                err text STORAGE PLAIN,
                message text STORAGE PLAIN
                -- model_transaction_blockdata
            );
            TRUNCATE transaction_raw_blockdata;
        "#;
        postgres_session.execute_multiple(statmement).await?;

        let statement = r#"
            COPY transaction_raw_blockdata(
                signature,
                slot,
                cu_requested,
                prioritization_fees,
                cu_consumed,
                recent_blockhash,
                err,
                message
                -- model_transaction_blockdata
            ) FROM STDIN BINARY
        "#;
        let started_at = Instant::now();
        let sink: CopyInSink<bytes::Bytes> = postgres_session.copy_in(statement).await?;
        let writer = BinaryCopyInWriter::new(
            sink,
            &[
                Type::TEXT,
                Type::INT8,
                Type::INT8,
                Type::INT8,
                Type::INT8,
                Type::TEXT,
                Type::TEXT,
                Type::TEXT, // model_transaction_blockdata
            ],
        );
        pin_mut!(writer);

        for tx in transactions {
            let PostgresTransaction {
                signature,
                slot,
                cu_requested,
                prioritization_fees,
                cu_consumed,
                err,
                recent_blockhash,
                message,
                // model_transaction_blockdata
            } = tx;

            writer
                .as_mut()
                .write(&[
                    &signature,
                    &slot,
                    &cu_requested,
                    &prioritization_fees,
                    &cu_consumed,
                    &err,
                    &recent_blockhash,
                    &message,
                    // model_transaction_blockdata
                ])
                .await?;
        }

        let num_rows = writer.finish().await?;
        debug!(
            "inserted {} raw transaction data rows into temp table in {}ms",
            num_rows,
            started_at.elapsed().as_millis()
        );

        let statement = format!(
            r#"
            INSERT INTO {schema}.transaction_ids(signature)
            SELECT signature from transaction_raw_blockdata
            ON CONFLICT DO NOTHING
            "#,
        );
        let started_at = Instant::now();
        let num_rows = postgres_session.execute(statement.as_str(), &[]).await?;
        debug!(
            "inserted {} signatures into transaction_ids table in {}ms",
            num_rows,
            started_at.elapsed().as_millis()
        );

        let statement = format!(
            r#"
                INSERT INTO {schema}.transaction_blockdata
                SELECT
                    ( SELECT transaction_id FROM {schema}.transaction_ids tx_lkup WHERE tx_lkup.signature = transaction_raw_blockdata.signature ),
                    slot,
                    cu_requested,
                    prioritization_fees,
                    cu_consumed,
                    err,
                    recent_blockhash,
                    message
                    -- model_transaction_blockdata
                FROM transaction_raw_blockdata
        "#,
            schema = schema,
        );
        let started_at = Instant::now();
        let num_rows = postgres_session.execute(statement.as_str(), &[]).await?;
        debug!(
            "inserted {} rows into transaction block table in {}ms",
            num_rows,
            started_at.elapsed().as_millis()
        );

        Ok(())
    }

    pub fn build_query_statement(epoch: EpochRef, slot: Slot) -> String {
        format!(
            r#"
                SELECT
                    (SELECT signature FROM {schema}.transaction_ids tx_ids WHERE tx_ids.transaction_id = transaction_blockdata.transaction_id),
                    cu_requested,
                    prioritization_fees,
                    cu_consumed,
                    err,
                    recent_blockhash,
                    message
                    -- model_transaction_blockdata
                FROM {schema}.transaction_blockdata
                WHERE slot = {}
            "#,
            slot,
            schema = PostgresEpoch::build_schema_name(epoch),
        )
    }
}
