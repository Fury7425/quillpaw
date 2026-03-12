use crate::models::AiProposal;
use uuid::Uuid;

pub async fn make_proposal(proposal_type: &str, title: &str, content: &str) -> Result<AiProposal, String> {
    Ok(AiProposal {
        id: Uuid::new_v4().to_string(),
        proposal_type: proposal_type.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        target_path: None,
        metadata: None,
    })
}
