#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub skill_identity: String,
    pub body_identity: String,
    pub hosts: Vec<String>,
    pub trusted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisclosedBody {
    pub name: String,
    pub skill_identity: String,
    pub body_identity: String,
    pub body: String,
    pub references: Vec<DisclosedReference>,
    pub grants_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisclosedReference {
    pub content_id: String,
    pub content_identity: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallReceipt {
    pub schema: String,
    pub pack_identity: String,
    pub skill_identity: String,
    pub body_identity: String,
    pub host: String,
    pub grants_authority: bool,
}

pub fn metadata(candidate: &super::model::Candidate) -> Result<Vec<Metadata>, super::Error> {
    let mut result = Vec::with_capacity(candidate.manifest.skills.len());
    for skill in &candidate.manifest.skills {
        let body = content(candidate, &skill.body_content_id)?;
        result.push(Metadata {
            name: skill.name.clone(),
            description: skill.description.clone(),
            skill_identity: super::identity::skill_identity(skill)?,
            body_identity: body.content_identity.clone(),
            hosts: skill.hosts.clone(),
            trusted: skill.trust == "trusted",
        });
    }
    Ok(result)
}

// r[impl lattice.operation_packs.skills.denied]
pub fn validate_body(admitted: &super::model::AdmittedPack, name: &str, body: &[u8]) -> Result<(), super::Error> {
    let skill = admitted
        .candidate
        .manifest
        .skills
        .iter()
        .find(|skill| skill.name == name)
        .ok_or(super::Error::Invalid("skill name"))?;
    if skill.trust != "trusted" {
        return Err(super::Error::Invalid("untrusted skill"));
    }
    let declared = content(&admitted.candidate, &skill.body_content_id)?;
    if super::identity::content_identity(body) != declared.content_identity {
        return Err(super::Error::Invalid("skill body identity"));
    }
    Ok(())
}

pub fn route_query(skill: &super::model::SkillManifest, query: &str) -> bool {
    let normalized = query.to_ascii_lowercase();
    let has_negative_route = skill.negative_routes.iter().any(|route| normalized.contains(&route.to_ascii_lowercase()));
    let has_positive_route = skill.positive_routes.iter().any(|route| normalized.contains(&route.to_ascii_lowercase()));
    has_positive_route && !has_negative_route
}

pub fn validate_routing_cases(skill: &super::model::SkillManifest) -> bool {
    !skill.positive_routes.is_empty()
        && !skill.negative_routes.is_empty()
        && skill.positive_routes.iter().all(|query| route_query(skill, query))
        && skill.negative_routes.iter().all(|query| !route_query(skill, query))
}

fn content<'a>(
    candidate: &'a super::model::Candidate,
    content_id: &str,
) -> Result<&'a super::model::ContentItem, super::Error> {
    candidate
        .manifest
        .contents
        .iter()
        .find(|content| content.id == content_id)
        .ok_or(super::Error::Invalid("skill content"))
}
