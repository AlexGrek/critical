use crit_shared::entities::{Invite, User};
use gitops_lib::store::GenericDatabaseProvider;

use crate::{errors::AppError, state::AppState, utils};

pub async fn use_registration_invite(state: &AppState, invite_id: &str, invite_key: &str) -> Result<(), AppError> {
    let invite_option = state.store.provider::<Invite>().try_get_by_key(invite_id).await?;
    match invite_option {
        Some(mut invite) => {
            if invite.used {
                return Err(AppError::Forbidden)
            }
            if invite_key != invite.invite_key {
                return Err(AppError::InvalidData(format!("Incorrect invite key")))
            }
            invite.used = true;
            state.store.provider::<Invite>().upsert(&invite).await?;
            return Ok(())
        },
        None => return Err(AppError::Forbidden)
    }
}

pub fn is_user_invite_issuer(user: &User) -> bool {
    return user.has_admin_status
}

fn generate_invite() -> Invite {
    return Invite {
        invite_uid: utils::generate_random_string(5),
        invite_key: utils::generate_random_string(18),
        used: false,
    }
}

pub async fn create_invite(state: &AppState, user: &User) -> Result<Invite, AppError> {
    if !is_user_invite_issuer(user) {
        return Err(AppError::Forbidden)
    }
    let invite = generate_invite();
    state.store.provider::<Invite>().insert(&invite).await?;
    Ok(invite)
}
