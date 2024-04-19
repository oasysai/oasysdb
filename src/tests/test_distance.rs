use super::*;

#[test]
fn distance_calculation() {
    let a = Vector::from(vec![1.0, 3.0, 5.0]);
    let b = Vector::from(vec![2.0, 4.0, 6.0]);

    let dot = Distance::Dot.calculate(&a, &b);
    let euclidean = Distance::Euclidean.calculate(&a, &b);
    let cosine = Distance::Cosine.calculate(&a, &b);
    let norm_cosine = Distance::NormCosine.calculate(&a, &b);

    assert_eq!(dot, 44.0);
    assert_eq!(euclidean, 1.7320508);
    assert_eq!(cosine, 0.99385864);
    assert_eq!(dot, norm_cosine);
}
