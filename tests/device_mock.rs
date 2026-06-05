//! End-to-end checks of the high-level controller against a mock transport —
//! the bytes that *would* hit the wire, verified without any hardware.

use mystic_light::protocol::{REPORT_ID, REPORT_LEN};
use mystic_light::transport::MockTransport;
use mystic_light::{Header, MysticLight, Rgb};

#[test]
fn fill_produces_a_well_formed_report() {
    let mut ml = MysticLight::new(MockTransport::new());
    ml.fill(Header::Jargb2, Rgb::new(0x0a, 0x1a, 0x2a), 3)
        .unwrap();

    let report = ml.transport().last().expect("a report was sent").to_vec();

    assert_eq!(report.len(), REPORT_LEN);
    assert_eq!(report[0], REPORT_ID);
    assert_eq!(report[1], 0x09);
    assert_eq!(report[2], 0x04);
    assert_eq!(report[3], Header::Jargb2.index());
    assert_eq!(report[6], 3);
    assert_eq!(
        &report[7..16],
        &[0x0a, 0x1a, 0x2a, 0x0a, 0x1a, 0x2a, 0x0a, 0x1a, 0x2a]
    );
    // The tail past the three LEDs is padding.
    assert!(report[16..].iter().all(|&b| b == 0));
}

#[test]
fn per_led_animation_frame() {
    let mut ml = MysticLight::new(MockTransport::new());
    let frame: Vec<Rgb> = (0..60)
        .map(|i| Rgb::from_hsv(i as f32 * 6.0, 1.0, 1.0))
        .collect();
    ml.set_leds(Header::Jargb3, &frame).unwrap();

    let report = ml.transport().last().unwrap();
    assert_eq!(report[6], 60);
    // First LED is pure red (hue 0).
    assert_eq!(&report[7..10], &[255, 0, 0]);
}
