use super::*;
use pp_core::Screen;
use proptest::prelude::*;

#[derive(serde::Deserialize)]
struct GotoParams {
    screen: Screen,
}

fn test_registry() -> Registry {
    let mut r = Registry::builtin();
    r.register(CommandSpec {
        id: "test.goto",
        label: "Go to",
        menu: "",
        shortcut: None,
        params: "{ screen: Screen }",
        undoable: true,
        enabled: always_enabled,
        run: |s, p| {
            let p: GotoParams = params("test.goto", p)?;
            s.doc.screen = p.screen;
            Ok(Value::Null)
        },
    });
    r.register(CommandSpec {
        id: "test.panic_after_change",
        label: "Panic",
        menu: "",
        shortcut: None,
        params: "",
        undoable: true,
        enabled: always_enabled,
        run: |s, _| {
            s.doc.screen = Screen::About;
            panic!("boom");
        },
    });
    r.register(CommandSpec {
        id: "test.fail_after_change",
        label: "Fail",
        menu: "",
        shortcut: None,
        params: "",
        undoable: true,
        enabled: always_enabled,
        run: |s, _| {
            s.doc.screen = Screen::About;
            Err(EngineError::Failed {
                id: "test.fail_after_change".into(),
                msg: "nope".into(),
            })
        },
    });
    r
}

fn engine() -> Engine {
    Engine::with_registry(test_registry())
}

#[test]
fn unknown_command_is_an_error() {
    let mut e = engine();
    assert_eq!(
        e.run("no.such", Value::Null),
        Err(EngineError::UnknownCommand("no.such".into()))
    );
}

#[test]
fn run_changes_doc_and_records_history() {
    let mut e = engine();
    e.run("test.goto", json!({"screen": "pattern_test"}))
        .unwrap();
    assert_eq!(e.doc().screen, Screen::PatternTest);
    assert_eq!(e.session().history.undo_len(), 1);
}

#[test]
fn no_op_change_is_not_recorded() {
    let mut e = engine();
    e.run("test.goto", json!({"screen": "main"})).unwrap();
    assert_eq!(e.session().history.undo_len(), 0);
}

#[test]
fn undo_redo_round_trip() {
    let mut e = Engine::new();
    assert!(matches!(
        e.run("edit.undo", Value::Null),
        Err(EngineError::Disabled { .. })
    ));
    e.run("refresh.speed", json!({"speed": 2})).unwrap();
    e.run("refresh.speed", json!({"speed": 9})).unwrap();
    e.run("edit.undo", Value::Null).unwrap();
    assert_eq!(e.doc().refresh_speed.get(), 2);
    e.run("edit.undo", Value::Null).unwrap();
    assert_eq!(e.doc().refresh_speed.get(), 7);
    e.run("edit.redo", Value::Null).unwrap();
    assert_eq!(e.doc().refresh_speed.get(), 2);
    // A new change clears redo.
    e.run("refresh.speed", json!({"speed": 4})).unwrap();
    assert!(e.enabled("edit.redo").is_err());
}

#[test]
fn undo_keeps_the_current_screen() {
    let mut e = engine();
    e.run("test.goto", json!({"screen": "motion_test"}))
        .unwrap();
    e.run("test.goto", json!({"screen": "about"})).unwrap();
    e.run("edit.undo", Value::Null).unwrap();
    assert_eq!(e.doc().screen, Screen::About);
}

#[test]
fn bad_params_leave_doc_untouched() {
    let mut e = engine();
    let err = e.run("test.goto", json!({"screen": 42})).unwrap_err();
    assert!(matches!(err, EngineError::BadParams { .. }));
    assert_eq!(e.doc(), &Doc::default());
}

#[test]
fn error_rolls_back() {
    let mut e = engine();
    let err = e.run("test.fail_after_change", Value::Null).unwrap_err();
    assert!(matches!(err, EngineError::Failed { .. }));
    assert_eq!(e.doc(), &Doc::default());
    assert_eq!(e.session().history.undo_len(), 0);
}

#[test]
fn panic_rolls_back() {
    let mut e = engine();
    let err = e.run("test.panic_after_change", Value::Null).unwrap_err();
    assert_eq!(err, EngineError::Panicked("test.panic_after_change".into()));
    assert_eq!(e.doc(), &Doc::default());
    // The engine stays usable after a caught panic.
    e.run("test.goto", json!({"screen": "about"})).unwrap();
    assert_eq!(e.doc().screen, Screen::About);
}

#[test]
fn history_is_capped() {
    let mut h = History::default();
    for _ in 0..history::HISTORY_CAP + 50 {
        h.record(Doc::default());
    }
    assert_eq!(h.undo_len(), history::HISTORY_CAP);
}

#[test]
fn builtin_specs_have_labels() {
    let r = Registry::builtin();
    assert!(r.iter().all(|c| !c.label.is_empty() && !c.id.is_empty()));
}

fn arb_json() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::from),
        any::<i64>().prop_map(Value::from),
        any::<f64>().prop_map(Value::from),
        ".{0,12}".prop_map(Value::from),
    ];
    leaf.prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::from),
            prop::collection::btree_map(".{0,8}", inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

proptest! {
    /// Untrusted ids and params never panic, and a failed run never changes the document.
    #[test]
    fn arbitrary_requests_are_safe(
        id in prop_oneof![
            Just("test.goto".to_owned()),
            Just("edit.undo".to_owned()),
            Just("edit.redo".to_owned()),
            ".{0,16}",
        ],
        params in arb_json(),
    ) {
        let mut e = engine();
        let before = e.doc().clone();
        if e.run(&id, params).is_err() {
            prop_assert_eq!(e.doc(), &before);
        }
    }
}
