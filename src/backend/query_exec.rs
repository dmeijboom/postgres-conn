use crate::proto::messages::{CommandComplete, CommandTag, ErrorResponse};

pub type QueryResult = Result<CommandComplete, ErrorResponse>;

pub trait QueryExec {
    fn execute(&self, query: &str) -> QueryResult;
}

pub struct NoopQueryExec {}

impl NoopQueryExec {
    pub fn new() -> Self {
        Self {}
    }
}

impl QueryExec for NoopQueryExec {
    fn execute(&self, _query: &str) -> QueryResult {
        Ok(CommandComplete {
            command_tag: CommandTag::Select(0),
        })
    }
}
