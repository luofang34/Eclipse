use super::*;

#[test]
fn reserved_characters_are_escaped() {
    assert_eq!(escape("a<b & \"c\">"), "a&lt;b &amp; &quot;c&quot;&gt;");
}

#[test]
fn each_frame_shows_for_its_share_of_the_loop() {
    let animation = Animation {
        frames: 4,
        seconds: 8.0,
    };
    assert!(
        animation
            .only_during(1)
            .contains("keyTimes=\"0;0.25000;0.50000\"")
    );
    assert_eq!(
        animation.instants(0.0, 1.0),
        vec![0.0, 0.25, 0.5, 0.75, 1.0]
    );
}
