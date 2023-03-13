use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn current_unix() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

pub fn normalize_vec(nums: Vec<i32>, target: i32) -> Vec<i32> {
    let tot: i32 = nums.iter().sum();

    let mut normalized = vec![];

    let diff = target as f32 / tot as f32;
    //assert_eq!(diff, 2342.);
    for num in nums {
        normalized.push((num as f32 * diff) as i32);
    }

    let diff = normalized.iter().sum::<i32>() - target;
    let positive = diff > 0;
    let diff = diff.abs();

    let veclen = normalized.len();
    for i in 0..diff {
        match positive {
            true => normalized[i as usize % veclen] -= 1,
            false => normalized[i as usize % veclen] += 1,
        }
    }

    normalized
}

/*
mod tests {
    use super::*;

    #[test]
    fn test_normalize_vec() {
        let some_vec = vec![5, 3, 2, 1];
        //        let some_vec = vec![5, 25];
        let _new_one = normalize_vec(some_vec, 100);
    }
}
*/
