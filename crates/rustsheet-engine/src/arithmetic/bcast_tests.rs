use super::bcast_get;

#[test]
fn bcast_get_broadcasts_each_axis_by_the_addressed_row() {
    let grid = vec![vec![1, 2, 3], vec![4], vec![5, 6]];
    assert_eq!(bcast_get(&grid, 0, 2), Some(&3));
    assert_eq!(bcast_get(&grid, 1, 2), Some(&4)); // a one-wide row repeats
    assert_eq!(bcast_get(&grid, 2, 2), None); // ragged: past the row's length
    assert_eq!(bcast_get(&grid, 3, 0), None);
    let one_row = vec![vec![7, 8]];
    assert_eq!(bcast_get(&one_row, 5, 1), Some(&8)); // a one-tall grid repeats
    assert_eq!(bcast_get::<i32>(&[], 0, 0), None);
}
