use crate::{
    error::Result,
    tool::exporters::{export_organization_vault, export_vault, ExportFormat},
    vault::{Cipher, Collection, Folder},
    Client,
};

pub struct ClientExporters<'a> {
    pub(crate) _client: &'a crate::Client,
}

impl<'a> ClientExporters<'a> {
    pub async fn export_vault(
        &self,
        folders: Vec<Folder>,
        ciphers: Vec<Cipher>,
        format: ExportFormat,
    ) -> Result<String> {
        export_vault(folders, ciphers, format)
    }

    pub async fn export_organization_vault(
        &self,
        collections: Vec<Collection>,
        ciphers: Vec<Cipher>,
        format: ExportFormat,
    ) -> Result<String> {
        export_organization_vault(collections, ciphers, format)
    }
}

impl<'a> Client {
    pub fn exporters(&'a self) -> ClientExporters<'a> {
        ClientExporters { _client: self }
    }
}
