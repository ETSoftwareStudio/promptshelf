use super::*;

fn open(library: &str, id: &str) -> Link {
    Link::Open {
        library: library.into(),
        id: id.into(),
    }
}

fn reason(link: Link) -> String {
    match link {
        Link::Bad { reason } => reason,
        other => panic!("expected a refused link, got {other:?}"),
    }
}

#[test]
fn open_and_fill_name_a_prompt() {
    assert_eq!(
        parse("promptshelf://open/personal/welcome"),
        open("personal", "welcome")
    );
    assert_eq!(
        parse("promptshelf://fill/acme/coroutine-audit"),
        Link::Fill {
            library: "acme".into(),
            id: "coroutine-audit".into()
        }
    );
}

#[test]
fn scheme_and_route_ignore_case_and_extra_slashes() {
    assert_eq!(
        parse("PromptShelf://OPEN/personal/welcome/"),
        open("personal", "welcome")
    );
    assert_eq!(
        parse("promptshelf:open/personal/welcome#top"),
        open("personal", "welcome")
    );
}

#[test]
fn an_empty_link_only_opens_the_window() {
    assert_eq!(parse("promptshelf://"), Link::Home);
    assert_eq!(parse("promptshelf:"), Link::Home);
    assert_eq!(parse("promptshelf:///?x=1"), Link::Home);
}

#[test]
fn ids_follow_the_core_rules() {
    for url in [
        "promptshelf://open/Personal/welcome",
        "promptshelf://open/personal/we%6Ccome",
        "promptshelf://open/personal/-welcome",
        "promptshelf://fill/personal/a--b",
        "promptshelf://open/personal/welcome.md",
    ] {
        assert_eq!(reason(parse(url)), "the prompt ID is not valid", "{url}");
    }
    let long = format!("promptshelf://open/personal/{}", "a".repeat(101));
    assert_eq!(reason(parse(&long)), "the prompt ID is not valid");
}

#[test]
fn open_needs_exactly_a_library_and_an_id() {
    for url in [
        "promptshelf://open",
        "promptshelf://open/welcome",
        "promptshelf://fill/personal/templates/welcome",
        "promptshelf://open/personal/../config",
    ] {
        assert!(
            reason(parse(url)).contains("needs a library and an ID"),
            "{url}"
        );
    }
}

#[test]
fn search_decodes_its_text() {
    assert_eq!(
        parse("promptshelf://search?q=release%20notes+v2&x=1"),
        Link::Search {
            query: "release notes v2".into()
        }
    );
    assert_eq!(
        parse("promptshelf://search?q=caf%C3%A9"),
        Link::Search {
            query: "café".into()
        }
    );
    assert_eq!(
        parse("promptshelf://search"),
        Link::Search { query: "".into() }
    );
}

#[test]
fn text_becomes_one_clean_line() {
    assert_eq!(
        parse("promptshelf://search?q=%0Arm%09-rf%00%20%20x"),
        Link::Search {
            query: "rm -rf x".into()
        }
    );
}

#[test]
fn bad_text_is_refused() {
    for url in [
        "promptshelf://search?q=%zz",
        "promptshelf://search?q=%+1",
        "promptshelf://search?q=%4",
        "promptshelf://search?q=%FF",
    ] {
        assert!(reason(parse(url)).contains("%-code"), "{url}");
    }
    let long = format!("promptshelf://search?q={}", "a".repeat(MAX_TEXT + 1));
    assert!(reason(parse(&long)).contains("longer than"));
}

#[test]
fn new_takes_an_optional_title() {
    assert_eq!(parse("promptshelf://new"), Link::New { title: None });
    assert_eq!(parse("promptshelf://new?title="), Link::New { title: None });
    assert_eq!(
        parse("promptshelf://new?title=Strict+review"),
        Link::New {
            title: Some("Strict review".into())
        }
    );
    assert!(reason(parse("promptshelf://new/x")).contains("nothing can follow"));
}

#[test]
fn other_routes_are_not_supported_yet() {
    assert_eq!(
        reason(parse("promptshelf://import?url=https://example.com/x.git")),
        "\"import\" links are not supported yet"
    );
    assert_eq!(
        reason(parse("promptshelf://delete/personal/welcome")),
        "\"delete\" links are not supported yet"
    );
    assert_eq!(
        reason(parse("promptshelf://%3Cscript%3E")),
        "this kind of link is not supported yet"
    );
}

#[test]
fn other_schemes_and_long_links_are_refused() {
    assert!(reason(parse("https://example.com/open/personal/welcome")).contains("not a"));
    assert!(reason(parse("promptshelfx://open/personal/welcome")).contains("not a"));
    assert!(reason(parse("welcome")).contains("not a"));
    let long = format!("promptshelf://search?q={}", "a".repeat(MAX_LINK));
    assert_eq!(reason(parse(&long)), "the link is too long");
}

#[test]
fn copy_link_round_trips() {
    let link = open_link("personal", "strict-pr-review");
    assert_eq!(link, "promptshelf://open/personal/strict-pr-review");
    assert_eq!(parse(&link), open("personal", "strict-pr-review"));
}

#[test]
fn the_ui_gets_a_route_field() {
    let json = serde_json::to_value(open("personal", "welcome")).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "route": "open", "library": "personal", "id": "welcome" })
    );
    let json = serde_json::to_value(Link::New { title: None }).unwrap();
    assert_eq!(json, serde_json::json!({ "route": "new", "title": null }));
}
