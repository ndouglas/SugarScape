use super::super::draws::{draw_index, DrawSource, PcgDraws};
use super::super::movement::{
    ahead, directed_step, edge_target, normal_increment, search_step, turn,
};
use super::super::{Pos, Resource};
use super::{setup, Scripted};
use crate::config::FieldError;
use std::f64::consts::{FRAC_PI_4, PI, TAU};

#[test]
fn setup_rejects_multiple_original_inputs() {
    let mut s = setup();
    s.width = 2;
    s.agents = 0;
    s.parameters.p_search = f64::NAN;
    let errors = s.validate().unwrap_err();
    for field in ["width", "agents", "parameters.p_search"] {
        assert!(errors.iter().any(|e| e.field == field));
    }
}
#[test]
fn dimensions_and_agent_limits_reject_original_values() {
    for n in [0, 2, 126, u32::MAX] {
        let mut s = setup();
        s.width = n;
        s.height = n;
        let errors = s.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.field == "width"));
        assert!(errors.iter().any(|e| e.field == "height"));
    }
    for agents in [0, 257, u32::MAX] {
        let mut s = setup();
        s.agents = agents;
        assert!(s
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == "agents"));
    }
}
#[test]
fn valid_extreme_grids_round_trip_final_cell() {
    for (size, site) in [(3, 8), (125, 15624)] {
        let mut s = setup();
        s.width = size;
        s.height = size;
        s.agents = 256;
        s.validate().unwrap();
        let pos = Pos {
            x: size - 1,
            y: size - 1,
        };
        assert_eq!(s.site(pos), site);
        assert_eq!(s.position(site).unwrap(), pos);
        assert!(s.position(site + 1).is_err());
        assert!(s.position(u64::MAX).is_err());
    }
}
#[test]
fn invalid_geometry_cannot_decode_sites() {
    let mut s = setup();
    s.width = 0;
    assert!(s.position(0).is_err());
    s.width = u32::MAX;
    assert!(s.position(0).is_err());
}
#[test]
fn resource_errors_identify_index_and_preserve_full_ids() {
    let mut s = setup();
    s.resources = vec![
        Resource {
            id: u64::MAX,
            pos: Pos { x: 0, y: 0 },
        },
        Resource {
            id: u64::MAX,
            pos: Pos { x: 0, y: 0 },
        },
        Resource { id: 1, pos: s.nest },
        Resource {
            id: 2,
            pos: Pos { x: 5, y: 0 },
        },
    ];
    let errors = s.validate().unwrap_err();
    for field in [
        "resources[1].id",
        "resources[1].pos",
        "resources[2].pos",
        "resources[3].pos",
    ] {
        assert!(errors.iter().any(|e| e.field == field));
    }
    s.resources.truncate(1);
    s.validate().unwrap();
}
#[test]
fn resource_capacity_and_nest_are_checked() {
    let mut s = setup();
    s.width = 125;
    s.height = 125;
    s.nest = Pos { x: 124, y: 124 };
    s.resources = (0..257)
        .map(|id| Resource {
            id,
            pos: Pos {
                x: (id % 125) as u32,
                y: (id / 125) as u32,
            },
        })
        .collect();
    assert!(s
        .validate()
        .unwrap_err()
        .iter()
        .any(|e| e.field == "resources"));
    s.resources.pop();
    s.validate().unwrap();
    s.nest = Pos { x: 125, y: 0 };
    assert!(s.validate().unwrap_err().iter().any(|e| e.field == "nest"));
}
#[test]
fn all_parameter_errors_are_prefixed() {
    let mut s = setup();
    s.parameters = crate::foraging::CpfaParameters {
        p_search: 2.0,
        p_return: -1.0,
        omega: f64::INFINITY,
        lambda_informed: -1.0,
        lambda_fidelity: 257.0,
        lambda_publish: f64::NAN,
        lambda_waypoint: -1.0,
    };
    let errors = s.validate().unwrap_err();
    assert_eq!(errors.len(), 7);
    assert!(errors.iter().all(|e| e.field.starts_with("parameters.")));
}
#[test]
fn a_zero_turn_still_waits_one_tick() {
    let mut d = Scripted::new(&[0.5, 0.5]);
    assert_eq!(turn(0.0, 0.0, &mut d).unwrap(), (0.0, 1));
    assert_eq!(d.next, 2);
}
#[test]
fn large_normal_turn_clips_to_pi_and_four_waits() {
    let mut d = Scripted::new(&[0.5, 0.0]);
    assert_eq!(turn(0.0, 4.0 * PI, &mut d).unwrap(), (PI, 4));
}
#[test]
fn normal_uses_cosine_box_muller() {
    let mut d = Scripted::new(&[0.5, 0.0]);
    assert!((normal_increment(1.0, &mut d).unwrap() - 1.1774100225154747).abs() < 1e-12);
}
#[test]
fn tiny_negative_turn_wraps_into_half_open_range() {
    let mut d = Scripted::new(&[0.5, 0.5]);
    assert_eq!(turn(0.0, 1e-20, &mut d).unwrap(), (0.0, 1));
}
#[test]
fn invalid_angles_fail_before_draws() {
    for heading in [f64::NAN, -1.0, TAU, f64::INFINITY] {
        let mut d = Scripted::new(&[]);
        assert!(turn(heading, 0.0, &mut d).is_err());
        assert_eq!(d.next, 0);
        let mut h = heading;
        assert!(search_step(&setup(), Pos { x: 2, y: 2 }, &mut h, &mut d).is_err());
    }
    for stddev in [f64::NAN, -1.0, f64::INFINITY] {
        let mut d = Scripted::new(&[]);
        assert!(normal_increment(stddev, &mut d).is_err());
        assert_eq!(d.next, 0);
    }
}
#[test]
fn diagonal_search_is_not_four_neighbor_navigation() {
    let mut heading = FRAC_PI_4;
    let mut d = Scripted::new(&[]);
    assert_eq!(
        search_step(&setup(), Pos { x: 2, y: 2 }, &mut heading, &mut d).unwrap(),
        Pos { x: 3, y: 3 }
    );
    assert_eq!(d.next, 0);
    assert_eq!(ahead(&setup(), Pos { x: 0, y: 0 }, PI), None);
}
#[test]
fn boundary_retries_exactly_32_outward_proposals() {
    let mut heading = PI;
    let mut d = Scripted::new(&[0.5; 32]);
    assert!(search_step(&setup(), Pos { x: 0, y: 0 }, &mut heading, &mut d).is_err());
    assert_eq!(d.next, 32);
}
#[test]
fn boundary_can_replace_heading_with_legal_proposal() {
    let mut heading = PI;
    let mut d = Scripted::new(&[0.0]);
    assert_eq!(
        search_step(&setup(), Pos { x: 0, y: 0 }, &mut heading, &mut d).unwrap(),
        Pos { x: 1, y: 0 }
    );
    assert_eq!(heading, 0.0);
}
#[test]
fn directed_same_and_adjacent_targets_use_no_draw() {
    for target in [Pos { x: 2, y: 2 }, Pos { x: 3, y: 3 }] {
        let mut d = Scripted::new(&[]);
        assert_eq!(
            directed_step(&setup(), Pos { x: 2, y: 2 }, target, &mut d).unwrap(),
            target
        );
        assert_eq!(d.next, 0);
    }
}
#[test]
fn directed_scan_order_and_cumulative_boundaries_are_half_open() {
    let cases = [
        (0.0, Pos { x: 3, y: 1 }),
        (0.2697521433898179, Pos { x: 3, y: 2 }),
        (0.7302478566101821, Pos { x: 3, y: 3 }),
        (f64::from_bits(1.0f64.to_bits() - 1), Pos { x: 3, y: 3 }),
    ];
    for (draw, want) in cases {
        assert_eq!(
            directed_step(
                &setup(),
                Pos { x: 2, y: 2 },
                Pos { x: 4, y: 2 },
                &mut Scripted::new(&[draw])
            )
            .unwrap(),
            want
        );
    }
    assert_eq!(
        directed_step(
            &setup(),
            Pos { x: 0, y: 0 },
            Pos { x: 2, y: 0 },
            &mut Scripted::new(&[0.0])
        )
        .unwrap(),
        Pos { x: 1, y: 0 }
    );
}
#[test]
fn historical_edges_use_one_index_and_one_coordinate_draw() {
    let mut s = setup();
    s.width = 5;
    s.height = 3;
    for (edge, want) in [
        (0.0, Pos { x: 4, y: 0 }),
        (0.25, Pos { x: 0, y: 2 }),
        (0.5, Pos { x: 4, y: 2 }),
        (0.75, Pos { x: 4, y: 2 }),
    ] {
        let mut d = Scripted::new(&[edge, f64::from_bits(1.0f64.to_bits() - 1)]);
        assert_eq!(edge_target(&s, &mut d).unwrap(), want);
        assert_eq!(d.next, 2);
    }
}
#[test]
fn edge_target_can_equal_edge_nest_without_moving() {
    let mut s = setup();
    s.nest = Pos { x: 0, y: 0 };
    let mut d = Scripted::new(&[0.0, 0.0]);
    let target = edge_target(&s, &mut d).unwrap();
    assert_eq!(target, s.nest);
    assert_eq!(directed_step(&s, s.nest, target, &mut d).unwrap(), s.nest);
    assert_eq!(d.next, 2);
}
#[test]
fn integer_selection_rejects_zero_and_includes_last_index() {
    let mut d = Scripted::new(&[]);
    assert!(draw_index(&mut d, 0).is_err());
    assert_eq!(d.next, 0);
    assert_eq!(
        draw_index(
            &mut Scripted::new(&[f64::from_bits(1.0f64.to_bits() - 1)]),
            125
        )
        .unwrap(),
        124
    );
}
struct Unchecked(f64);
impl DrawSource for Unchecked {
    fn uniform(&mut self) -> Result<f64, Vec<FieldError>> {
        Ok(self.0)
    }
}
#[test]
fn consumers_reject_invalid_values_from_untrusted_draw_sources() {
    for value in [f64::NAN, f64::INFINITY, -0.1, 1.0] {
        assert!(draw_index(&mut Unchecked(value), 4).is_err());
        assert!(normal_increment(0.0, &mut Unchecked(value)).is_err());
        assert!(directed_step(
            &setup(),
            Pos { x: 2, y: 2 },
            Pos { x: 4, y: 2 },
            &mut Unchecked(value)
        )
        .is_err());
        let mut h = PI;
        assert!(search_step(&setup(), Pos { x: 0, y: 0 }, &mut h, &mut Unchecked(value)).is_err());
    }
}
#[test]
fn pcg_adapter_uses_existing_single_uniform_stream() {
    use rand::Rng;
    let mut expected = crate::rng::seeded(7);
    let mut actual = expected.clone();
    assert_eq!(
        PcgDraws(&mut actual).uniform().unwrap(),
        expected.gen::<f64>()
    );
}
