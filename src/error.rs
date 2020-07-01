#[derive(Debug)]
pub enum IpseClientError {
    NoneFile,
    IpfsResp(ipfs_api::response::Error),
    Substrate(substrate_subxt::Error),
}

impl From<ipfs_api::response::Error> for IpseError {
    fn from(err: ipfs_api::response::Error) -> Self {
        IpseError::IpfsResp(err)
    }
}

impl From<substrate_subxt::Error> for IpseError {
    fn from(err: substrate_subxt::Error) -> Self {
        IpseError::Substrate(err)
    }
}
