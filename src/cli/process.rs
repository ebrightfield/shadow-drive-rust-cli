use super::Command;
use itertools::Itertools;
use shadow_drive_cli::process_shadow_api_response;
use shadow_drive_cli::{wait_for_user_confirmation, FILE_UPLOAD_BATCH_SIZE};
use shadow_drive_rust::models::ShadowFile;
use shadow_drive_rust::{ShadowDriveClient, StorageAccountVersion};
use solana_sdk::signature::Signer;

impl Command {
    pub async fn process<T: Signer>(
        &self,
        signer: T,
        url: &str,
        skip_confirm: bool,
    ) -> anyhow::Result<()> {
        let signer_pubkey = signer.pubkey();
        println!("Signing with {:?}", signer_pubkey);
        println!("Sending RPC requests to {}", url);
        let client = ShadowDriveClient::new(signer, url);
        match self {
            Command::CreateStorageAccount { name, size } => {
                println!("Create Storage Account {}: {}", name, size);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client
                    .create_storage_account(name, size.clone(), StorageAccountVersion::v2())
                    .await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::DeleteStorageAccount { storage_account } => {
                println!("Delete Storage Account {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_storage_account(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::CancelDeleteStorageAccount { storage_account } => {
                println!(
                    "Cancellation of Delete Storage Account {}",
                    storage_account.to_string()
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.cancel_delete_storage_account(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::ClaimStake { storage_account } => {
                println!(
                    "Claim Stake on Storage Account {}",
                    storage_account.to_string()
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.claim_stake(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::ReduceStorage {
                storage_account,
                size,
            } => {
                println!(
                    "Reduce Storage Capacity {}: {}",
                    storage_account.to_string(),
                    size
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.reduce_storage(storage_account, size.clone()).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::AddStorage {
                storage_account,
                size,
            } => {
                println!("Increase Storage {}: {}", storage_account.to_string(), size);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.add_storage(storage_account, size.clone()).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::AddImmutableStorage {
                storage_account,
                size,
            } => {
                println!(
                    "Increase Immutable Storage {}: {}",
                    storage_account.to_string(),
                    size
                );
                wait_for_user_confirmation(skip_confirm)?;
                let response = client
                    .add_immutable_storage(storage_account, size.clone())
                    .await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::MakeStorageImmutable { storage_account } => {
                println!("Make Storage Immutable {}", storage_account.to_string());
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.make_storage_immutable(storage_account).await;

                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::GetStorageAccount { storage_account } => {
                println!("Get Storage Account {}", storage_account.to_string());
                let response = client.get_storage_account(storage_account).await;

                let act = process_shadow_api_response(response)?;
                println!("{:#?}", act);
            }
            Command::GetStorageAccounts { owner } => {
                let owner = owner.as_ref().unwrap_or(&signer_pubkey);
                println!("Get Storage Accounts Owned By {}", owner.to_string());
                let response = client.get_storage_accounts(owner).await;
                let accounts = process_shadow_api_response(response)?;
                println!("{:#?}", accounts);
            }
            Command::ListFiles { storage_account } => {
                println!(
                    "List Files for Storage Account {}",
                    storage_account.to_string()
                );
                let response = client.list_objects(storage_account).await;
                let files = process_shadow_api_response(response)?;
                println!("{:#?}", files);
            }
            Command::GetText {
                storage_account,
                file,
            } => {
                let location = shadow_drive_cli::drive_url(storage_account, file);
                let resp = shadow_drive_cli::get_text(&location).await?;
                let last_modified = shadow_drive_cli::last_modified(resp.headers())?;
                println!("Get Text at {}", &location);
                println!("Last Modified: {}", last_modified);
                println!("");
                println!("{}", resp.text().await?);
            }
            Command::DeleteFile {
                storage_account,
                file,
            } => {
                let location = shadow_drive_cli::drive_url(storage_account, file);
                println!("Delete file {}", &location);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.delete_file(storage_account, location.clone()).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::EditFile {
                storage_account,
                file,
            } => {
                let basename = shadow_drive_cli::acquire_basename(file);
                let shdw_file = ShadowFile::file(basename, file.clone());
                println!("Edit file {} {}", storage_account.to_string(), file);
                wait_for_user_confirmation(skip_confirm)?;
                let response = client.edit_file(storage_account, shdw_file).await;
                let resp = process_shadow_api_response(response)?;
                println!("{:#?}", resp);
            }
            Command::GetObjectData {
                storage_account,
                file,
            } => {
                let location = shadow_drive_cli::drive_url(storage_account, file);
                println!("Get object data {} {}", storage_account.to_string(), file);
                let response = client.get_object_data(&location).await;
                let data = process_shadow_api_response(response)?;
                println!("{:#?}", data);
            }
            Command::StoreFiles {
                batch_size,
                storage_account,
                files,
            } => {
                println!("Store Files {} {:#?}", storage_account.to_string(), files);
                println!(
                    "WARNING: This CLI does not add any encryption on its own. \
                The files in their current state become public as soon as they're uploaded."
                );
                wait_for_user_confirmation(skip_confirm)?;
                for chunk in files.into_iter().chunks(FILE_UPLOAD_BATCH_SIZE) {
                    let response = client
                        .store_files(
                            &storage_account,
                            chunk
                                .iter()
                                .map(|s| {
                                    let basename = shadow_drive_cli::acquire_basename(s);
                                    ShadowFile::file(basename, s.clone())
                                })
                                .collect(),
                        )
                        .await;
                    let resp = process_shadow_api_response(response)?;
                    println!("{:#?}", resp);
                }
            }
        }
        Ok(())
    }
}
