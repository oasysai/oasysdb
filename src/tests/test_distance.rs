use super::*;

#[test]
fn distance_calculation() {
    let a = Vector::from(vec![1.0, 3.0, 5.0]);
    let b = Vector::from(vec![2.0, 4.0, 6.0]);

    let euclidean = Distance::Euclidean.calculate(&a, &b);
    let cosine = Distance::Cosine.calculate(&a, &b);

    assert_eq!(euclidean, 1.7320508);

    // When utilizing SIMD, the cosine distance is approximated.
    // So we just need to make sure the result is within a certain range.
    let diff = cosine - 0.00614136;
    assert!(diff < 0.01);
}
