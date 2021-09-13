use crate::{Iroha, Transaction};

pub struct Client<'a>(&'a Iroha);

impl<'a> Client<'a> {
    pub fn new(iroha: &'a Iroha) -> Self {
        Client(iroha)
    }

    pub fn submit_transaction(
        &self,
        transaction: &Transaction,
        account_name: &str,
    ) -> color_eyre::Result<()> {
        transaction.execute(self.0, account_name)
    }
}
