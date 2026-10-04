//! Category classification for skills.
//!
//! A skill's category comes from its `category:` frontmatter field when present; otherwise this
//! module infers one from the skill's name and description. Inference counts whole-word keyword
//! hits per category (so `details` never trips `ai`) and the category with the most hits wins,
//! ties breaking toward `KEYWORDS` order — specific topics before broad ones. No hits means
//! `other`.

const KEYWORDS: &[(&str, &[&str])] = &[
    ("security", &["security", "vulnerability", "vulnerabilities", "pentest", "penetration", "owasp", "exploit", "threat", "firewall", "captcha", "waf"]),
    ("data", &["database", "sql", "bigquery", "postgres", "postgresql", "mysql", "sqlite", "alloydb", "analytics", "dataset", "query", "queries", "spreadsheet", "excel", "csv"]),
    ("ai", &["ai", "llm", "model", "prompt", "rag", "embedding", "gemini", "gpt", "claude", "openai", "inference", "eval", "tuning", "mcp"]),
    ("devops", &["deploy", "deployment", "docker", "kubernetes", "k8s", "terraform", "infrastructure", "pipeline", "observability", "monitoring", "cloud", "serverless", "devops", "cloudflare", "workers", "wrangler", "hosting"]),
    ("research", &["research", "analysis", "analyses", "evaluate", "evaluation", "survey", "study", "studies", "compare", "comparison"]),
    ("coding", &["code", "codebase", "programming", "implement", "implementation", "debug", "bug", "refactor", "test", "tdd", "api", "repo", "repository", "git", "github", "lint", "typecheck", "compile", "backend", "frontend", "fullstack", "architecture", "framework", "cli", "terminal", "shell", "python", "typescript", "javascript"]),
    ("writing", &["writing", "copywriting", "documentation", "docs", "markdown", "essay", "blog", "content", "prd", "spec", "prose", "changelog", "readme"]),
    ("design", &["design", "ui", "ux", "animation", "layout", "prototype", "visual", "brand", "figma", "typography", "interface", "mockup", "wireframe", "aesthetic", "motion"]),
];
/// Whole-word match tolerant of plural and common verb endings, so `tests` matches `test`.
fn word_matches(word: &str, keyword: &str) -> bool {
    matches!(word.strip_prefix(keyword), Some("" | "s" | "es" | "ing" | "ed" | "d"))
}

pub(crate) fn infer(name: &str, description: &str) -> &'static str {
    let haystack = format!("{name} {description}").to_lowercase();
    let words: Vec<&str> = haystack
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    // Strictly-greater comparison keeps the earliest category on ties.
    let mut best: (&'static str, usize) = ("other", 0);
    for (category, keywords) in KEYWORDS {
        let hits = words.iter().filter(|word| keywords.iter().any(|keyword| word_matches(word, keyword))).count();
        if hits > best.1 { best = (category, hits); }
    }
    best.0
}
