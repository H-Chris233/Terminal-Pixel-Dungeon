
#[cfg(test)]
mod tests {
    use super::*;
    use bincode;

    #[test]
    fn test_enemy_kind_serialization() {
        let kind = EnemyKind::Dragon;
        let encoded = bincode::serialize(&kind).unwrap();
        let decoded: EnemyKind = bincode::deserialize(&encoded).unwrap();
        assert_eq!(kind, decoded);
    }
}
