use rust_app::{OwnershipModel, OwnershipTrainingExample, OwnershipTrainingSettings};

#[test]
fn trusted_binary_model_generalizes_and_roundtrips_its_estimates() {
    let examples: Vec<_> = (0..80)
        .map(|i| {
            let positive = i >= 40;
            OwnershipTrainingExample {
                features: vec![
                    if positive {
                        0.8 + (i % 5) as f64 * 0.02
                    } else {
                        0.05 + (i % 5) as f64 * 0.02
                    },
                    0.9,
                ],
                owns_speech: positive,
            }
        })
        .collect();
    let model = OwnershipModel::train(
        vec!["mapped_activity".into(), "raw_vad".into()],
        &examples,
        OwnershipTrainingSettings::default(),
        None,
    )
    .unwrap();
    assert!(model.estimate(&[0.75, 0.95]).unwrap() > 0.9);
    assert!(
        model.estimate(&[0.15, 0.95]).unwrap() < 0.1,
        "Speech presence alone must not prove microphone ownership"
    );
    let reloaded = OwnershipModel::from_json(&model.to_json().unwrap()).unwrap();
    assert!(
        (model.estimate(&[0.75, 0.95]).unwrap() - reloaded.estimate(&[0.75, 0.95]).unwrap()).abs()
            < 1e-12
    );
    assert!(model.diagnostics().gradient_norm < 1e-5);
    let reversed: Vec<_> = examples.into_iter().rev().collect();
    let repeat = OwnershipModel::train(
        vec!["mapped_activity".into(), "raw_vad".into()],
        &reversed,
        OwnershipTrainingSettings::default(),
        None,
    )
    .unwrap();
    assert!(
        (model.estimate(&[0.75, 0.95]).unwrap() - repeat.estimate(&[0.75, 0.95]).unwrap()).abs()
            < 1e-12
    );
}

#[test]
fn unrelated_imbalanced_truth_cannot_be_accepted_as_a_useful_owner_model() {
    let examples: Vec<_> = (0..100)
        .map(|i| OwnershipTrainingExample {
            features: vec![(i % 10) as f64],
            owns_speech: i >= 90,
        })
        .collect();
    assert!(
        OwnershipModel::train(
            vec!["unrelated".into()],
            &examples,
            OwnershipTrainingSettings::default(),
            None
        )
        .is_err(),
        "Learning only the class prior is not useful ownership evidence"
    );
}

#[test]
fn unusable_truth_and_unconverged_training_are_rejected_without_confident_output() {
    let names = vec!["evidence".to_string()];
    for examples in [
        vec![],
        vec![
            OwnershipTrainingExample {
                features: vec![0.9],
                owns_speech: true
            };
            8
        ],
        (0..8)
            .map(|i| OwnershipTrainingExample {
                features: vec![0.9],
                owns_speech: i >= 4,
            })
            .collect(),
        (0..8)
            .map(|i| OwnershipTrainingExample {
                features: vec![f64::NAN],
                owns_speech: i >= 4,
            })
            .collect(),
    ] {
        assert!(
            OwnershipModel::train(
                names.clone(),
                &examples,
                OwnershipTrainingSettings::default(),
                None
            )
            .is_err()
        );
    }
    let examples: Vec<_> = (0..80)
        .map(|i| OwnershipTrainingExample {
            features: vec![i as f64],
            owns_speech: i >= 40,
        })
        .collect();
    assert!(
        OwnershipModel::train(
            names.clone(),
            &examples,
            OwnershipTrainingSettings {
                max_iterations: 1,
                ..Default::default()
            },
            None
        )
        .is_err()
    );
    let model = OwnershipModel::train(names, &examples, OwnershipTrainingSettings::default(), None)
        .unwrap();
    assert!(model.estimate(&[f64::INFINITY]).is_err());
    assert!(model.estimate(&[]).is_err());
    let mut json: serde_json::Value = serde_json::from_str(&model.to_json().unwrap()).unwrap();
    json["settings"]["mean_loss_l2"] = serde_json::json!(-1.0);
    assert!(
        OwnershipModel::from_json(&json.to_string()).is_err(),
        "Reload cannot accept an impossible training recipe"
    );
}

#[test]
fn nonempty_saved_mask_cannot_drop_the_informative_training_dimension() {
    let examples: Vec<_> = (0..80)
        .map(|i| OwnershipTrainingExample {
            features: vec![if i >= 40 { 0.9 } else { 0.1 }, (i % 5) as f64],
            owns_speech: i >= 40,
        })
        .collect();
    let model = OwnershipModel::train(
        vec!["ownership_activity".into(), "unrelated_variation".into()],
        &examples,
        OwnershipTrainingSettings::default(),
        None,
    )
    .unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&model.to_json().unwrap()).unwrap();
    json["retained"] = serde_json::json!([1]);
    json["coefficients"] = serde_json::json!([0.0]);
    json["intercept"] = serde_json::json!(0.0);
    assert!(
        OwnershipModel::from_json(&json.to_string()).is_err(),
        "A nonempty prior-only mask cannot claim the trained full feature recipe"
    );
}
