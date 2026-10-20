//! Authored specimen copy (caption, explanation, citation) embedded from `assets/specimens`.
use bevy::prelude::Resource;
use serde::Deserialize;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/specimens/specimens.toml"
));

/// Library section a specimen belongs to.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    /// Grids and continuous fields.
    Cellular,
    /// Symbolic recognisers and Turing machines.
    Machines,
    /// Rewriting systems.
    Grammars,
    /// Multi-agent arenas.
    Agents,
}

impl Group {
    /// Library order.
    pub const ALL: [Self; 4] = [Self::Cellular, Self::Machines, Self::Grammars, Self::Agents];

    /// Margin label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Cellular => "cellular",
            Self::Machines => "machines",
            Self::Grammars => "grammars",
            Self::Agents => "agents",
        }
    }
}

/// One library entry.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Specimen {
    /// Catalog id passed to `AutomataCommand::SeedPattern`.
    pub id: String,
    /// Serif display title.
    pub title: String,
    /// Library section.
    pub group: Group,
    /// Italic library subtitle.
    pub subtitle: String,
    /// Stage figure caption.
    pub caption: String,
    /// One-line footnote explanation.
    pub explanation: String,
    /// Headline metric name (matches the backend's `MetricId::name`).
    pub metric: String,
    /// Full reference.
    pub citation: String,
    /// DOI without the resolver prefix.
    pub doi: String,
}

impl Specimen {
    /// Resolver URL for the DOI.
    pub fn doi_url(&self) -> String {
        format!("https://doi.org/{}", self.doi)
    }
}

#[derive(Deserialize)]
struct File {
    specimen: Vec<Specimen>,
}

/// Every specimen in file order.
#[derive(Resource, Clone, Debug)]
pub struct Specimens(pub Vec<Specimen>);

impl Specimens {
    /// Parse and validate TOML specimen data.
    pub fn parse(source: &str) -> Result<Self, String> {
        let file: File = toml::from_str(source).map_err(|e| e.to_string())?;
        if let Some(bad) = file.specimen.iter().find(|s| !valid_doi(&s.doi)) {
            return Err(format!("{}: malformed DOI {}", bad.id, bad.doi));
        }
        Ok(Self(file.specimen))
    }

    /// The compiled-in file; validity is guaranteed by this module's tests.
    pub fn embedded() -> Self {
        Self::parse(SOURCE).expect("assets/specimens/specimens.toml is validated by tests")
    }

    /// Entry for a catalog id; Lenia seed variants share the `lenia` entry.
    pub fn get(&self, id: &str) -> Option<&Specimen> {
        let id = if id.starts_with("lenia") { "lenia" } else { id };
        self.0.iter().find(|s| s.id == id)
    }

    /// 1-based figure number in file order.
    pub fn figure(&self, id: &str) -> Option<usize> {
        let id = self.get(id)?.id.as_str();
        self.0.iter().position(|s| s.id == id).map(|i| i + 1)
    }
}

/// `10.<4–9 digit registrant>/<non-empty suffix without whitespace>`.
pub fn valid_doi(doi: &str) -> bool {
    let Some((registrant, suffix)) = doi
        .strip_prefix("10.")
        .and_then(|rest| rest.split_once('/'))
    else {
        return false;
    };
    (4..=9).contains(&registrant.len())
        && registrant.chars().all(|c| c.is_ascii_digit())
        && !suffix.is_empty()
        && !suffix.chars().any(char::is_whitespace)
}
