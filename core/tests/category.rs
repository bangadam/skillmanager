use skillmanager_core::{scan, Agent, Root};
use std::{fs, path::Path};

fn skill(root: &Path, folder: &str, markdown: &str) {
    let dir = root.join(folder);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), markdown).unwrap();
}

fn categories(root: &Path) -> Vec<(String, String)> {
    scan(&[Root { agent: Agent::Omp, path: root.to_path_buf() }]).unwrap()
        .into_iter()
        .map(|skill| (skill.id, skill.category))
        .collect()
}

fn category_of(root: &Path, id: &str) -> String {
    categories(root).into_iter().find(|(skill_id, _)| skill_id == id).unwrap().1
}

#[test]
fn frontmatter_category_overrides_and_normalizes_inference() {
    let temp = tempfile::tempdir().unwrap();
    skill(temp.path(), "overrides", "---\nname: Overrides\ndescription: Debug this failing code\ncategory: Design \n---\nbody");
    skill(temp.path(), "blank-category", "---\ndescription: Debug this failing code\ncategory: \"  \"\n---\nbody");
    assert_eq!(category_of(temp.path(), "overrides"), "design");
    assert_eq!(category_of(temp.path(), "blank-category"), "coding");
}

#[test]
fn infers_each_category_from_name_and_description() {
    let temp = tempfile::tempdir().unwrap();
    skill(temp.path(), "tdd", "---\ndescription: Test-first feature and bug work\n---");
    skill(temp.path(), "brand-kit", "---\ndescription: Logo systems and identity decks\n---");
    skill(temp.path(), "vendor-scan", "---\ndescription: Survey and compare vendor systems\n---");
    skill(temp.path(), "release-notes", "---\ndescription: Changelog and readme writing\n---");
    skill(temp.path(), "report-gen", "---\ndescription: SQL query analytics over datasets\n---");
    skill(temp.path(), "model-garden", "---\ndescription: Deploy gemini models to endpoints\n---");
    skill(temp.path(), "ship-it", "---\ndescription: Kubernetes and terraform infrastructure\n---");
    skill(temp.path(), "vault", "---\ndescription: Finds OWASP vulnerabilities\n---");
    skill(temp.path(), "explain", "---\ndescription: Teaches concepts patiently\n---");
    let categories = categories(temp.path());
    let expected = [
        ("brand-kit", "design"),
        ("explain", "other"),
        ("model-garden", "ai"),
        ("release-notes", "writing"),
        ("report-gen", "data"),
        ("ship-it", "devops"),
        ("tdd", "coding"),
        ("vault", "security"),
        ("vendor-scan", "research"),
    ];
    for (id, expected) in expected {
        assert_eq!(categories.iter().find(|(skill_id, _)| skill_id == id).unwrap().1, expected, "{id}");
    }
}

#[test]
fn matches_whole_words_not_substrings() {
    let temp = tempfile::tempdir().unwrap();
    // "details" must not trip `ai`, "building" must not trip `ui`, "latest" must not trip `test`.
    skill(temp.path(), "subtle", "---\ndescription: Maintains available details in every latest guideline while building\n---");
    assert_eq!(category_of(temp.path(), "subtle"), "other");
}

#[test]
fn infers_from_folder_name_when_frontmatter_is_missing_or_malformed() {
    let temp = tempfile::tempdir().unwrap();
    skill(temp.path(), "code-gen", "No frontmatter at all");
    skill(temp.path(), "broken", "---\nname: [not closed\n---\nbody");
    assert_eq!(category_of(temp.path(), "code-gen"), "coding");
    // Malformed frontmatter degrades to id-based inference; "broken" matches no keyword.
    assert_eq!(category_of(temp.path(), "broken"), "other");
}

#[test]
fn first_copy_wins_like_name_and_description() {
    let temp = tempfile::tempdir().unwrap();
    let left = temp.path().join("left");
    let right = temp.path().join("right");
    fs::create_dir_all(&left).unwrap();
    fs::create_dir_all(&right).unwrap();
    skill(&left, "alpha", "---\ndescription: Design system tokens\n---");
    skill(&right, "alpha", "---\ndescription: Kubernetes deploy pipeline\ncategory: devops\n---");
    let result = scan(&[
        Root { agent: Agent::ClaudeCode, path: left },
        Root { agent: Agent::Codex, path: right },
    ]).unwrap();
    assert_eq!(result[0].category, "design");
}
