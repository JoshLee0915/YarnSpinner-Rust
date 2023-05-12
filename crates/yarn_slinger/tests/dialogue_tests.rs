//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner.Tests/DialogueTests.cs>
//!
//! ## Implementation notes
//! `TestDumpingCode` was not ported because `GetByteCode` is not used by a user directly and thus was not implemented at all.

use test_base::prelude::*;
use yarn_slinger::prelude::*;

mod test_base;

#[test]
fn test_node_exists() {
    let path = space_demo_scripts_path().join("Sally.yarn");
    let test_base = TestBase::new();

    let result = Compiler::new()
        .read_file(path)
        .extend_library(test_base.dialogue.library().clone())
        .compile()
        .unwrap();

    let mut dialogue = test_base.dialogue;
    dialogue.replace_program(result.program.unwrap());
    assert!(dialogue.node_exists("Sally"));

    // Test clearing everything
    dialogue.unload_all();
    assert!(!dialogue.node_exists("Sally"));
}

#[test]
fn test_analysis() {
    let mut context = Context::with_default_analysers();
    let test_base = TestBase::new();

    // this script has the following variables:
    // $foo is read from and written to
    // $bar is written to but never read
    // this means that there should be one diagnosis result
    let path = test_data_path().join("AnalysisTest.yarn");
    let result = Compiler::new()
        .read_file(path)
        .extend_library(test_base.dialogue.library().clone())
        .compile()
        .unwrap();

    test_base
        .with_compilation(result)
        .dialogue
        .analyse(&mut context);

    let diagnoses: Vec<_> = context
        .finish_analysis()
        .into_iter()
        .filter(|d| d.severity == DiagnosisSeverity::Warning)
        .collect();
    println!("{diagnoses:#?}");

    assert_eq!(1, diagnoses.len());
    assert!(diagnoses[0]
        .message
        .contains("Variable $bar is assigned, but never read from"));
}

#[test]
/// Split off from `test_analysis`
fn test_analysis_has_no_false_positives() {
    let test_base = TestBase::new();
    let result = Compiler::new()
        .read_file(space_demo_scripts_path().join("Sally.yarn"))
        .read_file(space_demo_scripts_path().join("Ship.yarn"))
        .extend_library(test_base.dialogue.library().clone())
        .compile()
        .unwrap();
    let mut context = Context::with_default_analysers();
    test_base
        .with_compilation(result)
        .dialogue
        .analyse(&mut context);

    let diagnoses: Vec<_> = context
        .finish_analysis()
        .into_iter()
        .filter(|d| d.severity == DiagnosisSeverity::Warning)
        .collect();
    println!("{diagnoses:#?}");
    assert!(diagnoses.is_empty());
}

#[test]
#[should_panic(expected = "No node named \"THIS NODE DOES NOT EXIST\" has been loaded.")]
fn test_missing_node() {
    let path = test_data_path().join("TestCases").join("Smileys.yarn");

    let result = Compiler::new().read_file(path).compile().unwrap();

    let mut test_base = TestBase::new()
        .with_program(result.program.unwrap())
        .with_runtime_errors_do_not_cause_failure();
    test_base.dialogue.set_node("THIS NODE DOES NOT EXIST");
}

#[test]
fn test_getting_current_node_name() {
    let path = space_demo_scripts_path().join("Sally.yarn");
    let test_base = TestBase::new();

    let result = Compiler::new()
        .read_file(path)
        .extend_library(test_base.dialogue.library().clone())
        .compile()
        .unwrap();

    let mut dialogue = test_base.dialogue;
    dialogue.replace_program(result.program.unwrap());

    // dialogue should not be running yet
    assert!(dialogue.current_node().is_none());

    dialogue.set_node("Sally");
    assert_eq!(dialogue.current_node(), Some("Sally".to_string()));

    dialogue.stop();
    // Current node should now be null
    assert!(dialogue.current_node().is_none());
}

#[test]
fn test_getting_raw_source() {
    let path = test_data_path().join("Example.yarn");
    let mut test_base = TestBase::new();

    let result = Compiler::new().read_file(path).compile().unwrap();

    test_base = test_base.with_compilation(result);
    let dialogue = &test_base.dialogue;

    let source_id = dialogue.get_string_id_for_node("LearnMore").unwrap();
    let source = test_base
        .string_table
        .get(&LineId(source_id))
        .unwrap()
        .text
        .clone();

    assert_eq!(source, "A: HAHAHA\n");
}

#[test]
fn test_getting_tags() {
    let path = test_data_path().join("Example.yarn");
    let mut test_base = TestBase::new();

    let result = Compiler::new().read_file(path).compile().unwrap();

    test_base = test_base.with_program(result.program.unwrap());
    let dialogue = &test_base.dialogue;

    let tags = dialogue.get_tags_for_node("LearnMore").unwrap();

    assert_eq!(tags, vec!["rawText"]);
}

/// ## Implementation note
/// Corresponds to `TestPrepareForLine`
#[test]
fn test_line_hints() {
    let path = test_data_path().join("TaggedLines.yarn");

    let result = Compiler::new().read_file(path).compile().unwrap();

    let mut dialogue = TestBase::new()
        .with_compilation(result)
        .dialogue
        .with_should_send_line_hints()
        .with_node_at_start();

    let mut line_hints_were_sent = false;

    let events = dialogue.next().unwrap();
    for event in events {
        if let DialogueEvent::LineHints(lines) = event {
            // When the Dialogue realises it's about to run the Start
            // node, it will tell us that it's about to run these two line IDs
            assert_eq!(lines.len(), 2);
            println!("{:?}", lines);
            assert!(lines.contains(&"line:test1".into()));
            assert!(lines.contains(&"line:test2".into()));

            // Ensure that these asserts were actually called
            line_hints_were_sent = true;
        }
    }

    assert!(line_hints_were_sent);
}

#[test]
fn test_function_argument_type_inference() {
    let test_base = TestBase::new().extend_library(
        Library::new()
            // Register some functions
            .with_function("ConcatString", |a: &str, b: &str| format!("{a}{b}"))
            .with_function("AddInt", |a: i32, b: i32| a + b)
            .with_function("AddFloat", |a: f32, b: f32| a + b)
            .with_function("NegateBool", |a: bool| !a),
    );

    // Run some code to exercise these functions
    let source = "\
    <<declare $str = \"\">>
    <<declare $int = 0>>
    <<declare $float = 0.0>>
    <<declare $bool = false>>

    <<set $str = ConcatString(\"a\", \"b\")>>
    <<set $int = AddInt(1,2)>>
    <<set $float = AddFloat(1,2)>>
    <<set $bool = NegateBool(true)>>
    ";

    let result = Compiler::from_test_source(source)
        .extend_library(test_base.dialogue.library().clone())
        .compile()
        .unwrap();

    let storage = test_base
        .with_compilation(result)
        .run_standard_testcase()
        .variable_store
        .clone_shallow();

    // The values should be of the right type and value
    let str_value: String = storage.get("$str").unwrap().into();
    assert_eq!("ab", &str_value);

    let int_value: i32 = storage.get("$int").unwrap().try_into().unwrap();
    assert_eq!(3, int_value);

    let float_value: f32 = storage.get("$float").unwrap().try_into().unwrap();
    assert_eq!(3.0, float_value);

    let bool_value: bool = storage.get("$bool").unwrap().try_into().unwrap();
    assert!(!bool_value);
}

#[test]
fn test_selecting_option_from_inside_option_callback() {
    let result = Compiler::from_test_source("-> option 1\n->option 2\nfinal line\n")
        .compile()
        .unwrap();

    let mut test_base = TestBase::new()
        .with_test_plan(
            TestPlan::new()
                .expect_option("option 1")
                .expect_option("option 2")
                .then_select(0)
                .expect_line("final line"),
        )
        .with_compilation(result);
    test_base.dialogue.set_node_to_start();

    while let Some(events) = test_base.dialogue.next() {
        for event in events {
            match event {
                DialogueEvent::Line(line) => {
                    let line_text = test_base.string_table.get(&line.id).unwrap().text.clone();
                    let parsed_text = test_base.dialogue.parse_markup(&line_text);
                    let test_plan = test_base.test_plan.as_mut().unwrap();
                    test_plan.next();

                    let expected_step = test_plan.next_expected_step;
                    let expected_value = test_plan.next_step_value.clone().unwrap();
                    assert_eq!(ExpectedStepType::Line, expected_step);
                    assert_eq!(StepValue::String(parsed_text), expected_value);
                }
                DialogueEvent::Options(options) => {
                    test_base.test_plan.as_mut().unwrap().next();
                    let actual_options: Vec<_> = options
                        .into_iter()
                        .map(|o| ProcessedOption {
                            line: test_base.get_composed_text_for_line(&o.line),
                            enabled: o.is_available,
                        })
                        .collect();
                    let test_plan = test_base.test_plan.as_ref().unwrap();
                    let next_expected_options = test_plan.next_expected_options.clone();
                    assert_eq!(next_expected_options, actual_options);

                    let expected_step = test_plan.next_expected_step;
                    assert_eq!(ExpectedStepType::Select, expected_step);
                    test_base
                        .dialogue
                        .set_selected_option(OptionId::construct_for_debugging(0));
                }
                DialogueEvent::DialogueComplete => {
                    let test_plan = test_base.test_plan.as_mut().unwrap();
                    test_plan.next();
                    let expected_step = test_plan.next_expected_step;
                    assert_eq!(ExpectedStepType::Stop, expected_step);
                }
                DialogueEvent::Command(_)
                | DialogueEvent::NodeComplete(_)
                | DialogueEvent::NodeStart(_)
                | DialogueEvent::LineHints(_) => {}
            }
        }
    }
}
