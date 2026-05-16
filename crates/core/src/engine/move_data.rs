use serde::{Deserialize, Serialize};
use std::fmt;

/// Parsed engine analysis data for a single candidate move.
/// Mirrors MoveData.java with all fields from Leela/KataGo/SAI formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveData {
    /// Move coordinate in engine format (e.g. "Q16", "D4")
    pub coordinate: String,
    /// Number of visits/playouts
    pub playouts: usize,
    /// Win rate as percentage (0-100)
    pub winrate: f64,
    /// Principal variation moves
    pub variation: Vec<String>,
    /// PV visits per move (KataGo)
    pub pv_visits: Vec<String>,
    /// Lower confidence bound as percentage (0-100)
    pub lcb: f64,
    /// Policy prior probability as percentage (0-100)
    pub policy: f64,
    /// Mean score estimate (from black's perspective for KataGo)
    pub score_mean: f64,
    /// Score standard deviation
    pub score_stdev: f64,
    /// Whether this data comes from KataGo
    pub is_kata_data: bool,
    /// Whether this data comes from SAI
    pub is_sai_data: bool,
    /// Display order
    pub order: i32,
    /// Whether this is the next move in the game
    pub is_next_move: bool,
    /// Best winrate seen so far
    pub best_winrate: f64,
    /// Best score mean seen so far
    pub best_score_mean: f64,
    /// Whether this is a symmetry duplicate
    pub is_symmetry: bool,
    /// Per-move ownership estimates (KataGo movesOwnership)
    pub moves_estimate_array: Vec<f64>,
}

impl Default for MoveData {
    fn default() -> Self {
        MoveData {
            coordinate: String::new(),
            playouts: 0,
            winrate: 0.0,
            variation: Vec::new(),
            pv_visits: Vec::new(),
            lcb: 0.0,
            policy: 0.0,
            score_mean: 0.0,
            score_stdev: 0.0,
            is_kata_data: false,
            is_sai_data: false,
            order: 0,
            is_next_move: false,
            best_winrate: 0.0,
            best_score_mean: 0.0,
            is_symmetry: false,
            moves_estimate_array: Vec::new(),
        }
    }
}

impl MoveData {
    /// Parse Leela Zero format: `info move Q16 visits 80 winrate 4405 prior 1828 lcb 4379 order 0 pv Q16 D4`
    /// Winrate/lcb/prior are integers divided by 100.
    pub fn from_info(line: &str) -> Option<Self> {
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() {
            return None;
        }

        let mut result = MoveData::default();
        let mut i = 0;

        // Skip "info" prefix if present
        if tokens[0] == "info" {
            i = 1;
        }

        while i < tokens.len() - 1 {
            let key = tokens[i];
            if key == "pv" {
                result.variation = tokens[i + 1..].iter().map(|s| s.to_string()).collect();
                break;
            }
            i += 1;
            let value = tokens[i];
            match key {
                "order" => result.order = value.parse().unwrap_or(0),
                "move" => result.coordinate = value.to_string(),
                "visits" => result.playouts = value.parse().unwrap_or(0),
                "lcb" => result.lcb = value.parse::<f64>().unwrap_or(0.0) / 100.0,
                "prior" => result.policy = value.parse::<f64>().unwrap_or(0.0) / 100.0,
                "winrate" => result.winrate = value.parse::<f64>().unwrap_or(0.0) / 100.0,
                _ => {}
            }
            i += 1;
        }

        result.is_kata_data = false;
        result.is_sai_data = false;
        Some(result)
    }

    /// Parse KataGo format: `info move D4 visits 100 winrate 0.4405 prior 0.1828 lcb 0.4379 order 0 pv D4 Q16 scoreMean -3.2 scoreStdev 22.1`
    /// Winrate/lcb/prior are floats in 0-1 range, multiplied by 100.
    pub fn from_info_katago(line: &str) -> Option<Self> {
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() {
            return None;
        }

        let mut result = MoveData::default();
        let mut pv_start = 0;
        let mut pv_visits_pos = 0;
        let mut moves_ownership_pos = 0;

        // Skip "info" prefix
        let start = if tokens[0] == "info" { 1 } else { 0 };

        // Find positions of special sections
        for idx in start..tokens.len() {
            match tokens[idx] {
                "pv" => pv_start = idx,
                "pvVisits" => pv_visits_pos = idx,
                "movesOwnership" => moves_ownership_pos = idx,
                _ => {}
            }
        }

        let mut i = start;

        // Parse key-value pairs before pv
        while i < tokens.len() - 1 {
            let key = tokens[i];
            if key == "pv" {
                break;
            }
            i += 1;
            let value = tokens[i];
            match key {
                "order" => result.order = value.parse().unwrap_or(0),
                "move" => result.coordinate = value.to_string(),
                "visits" => result.playouts = value.parse().unwrap_or(0),
                "lcb" => result.lcb = value.parse::<f64>().unwrap_or(0.0) * 100.0,
                "prior" => result.policy = value.parse::<f64>().unwrap_or(0.0) * 100.0,
                "winrate" => result.winrate = value.parse::<f64>().unwrap_or(0.0) * 100.0,
                "scoreMean" => result.score_mean = value.parse().unwrap_or(0.0),
                "scoreStdev" => result.score_stdev = value.parse().unwrap_or(0.0),
                "isSymmetryOf" => result.is_symmetry = true,
                _ => {}
            }
            i += 1;
        }

        // Parse variation
        if pv_start > 0 {
            let var_end = if pv_visits_pos > 0 {
                pv_visits_pos
            } else if moves_ownership_pos > 0 {
                moves_ownership_pos
            } else {
                tokens.len()
            };
            result.variation = tokens[pv_start + 1..var_end]
                .iter()
                .map(|s| s.to_string())
                .collect();

            if pv_visits_pos > 0 {
                let pv_end = if moves_ownership_pos > 0 {
                    moves_ownership_pos
                } else {
                    tokens.len()
                };
                result.pv_visits = tokens[pv_visits_pos + 1..pv_end]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                if result.pv_visits.len() > result.variation.len() {
                    result.pv_visits.truncate(result.variation.len());
                }
            }

            if moves_ownership_pos > 0 {
                for tok in &tokens[moves_ownership_pos + 1..] {
                    if let Ok(v) = tok.parse::<f64>() {
                        result.moves_estimate_array.push(v);
                    } else {
                        break;
                    }
                }
            }
        }

        result.is_kata_data = true;
        result.is_sai_data = false;
        Some(result)
    }

    /// Parse SAI format with `areas` or Sayuri's `scoreLead`.
    pub fn from_info_sai(line: &str, is_sayuri: bool, black_to_play: bool) -> Option<Self> {
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() {
            return None;
        }

        let mut result = MoveData::default();
        let mut i = 0;

        // Skip "info" prefix
        if tokens[0] == "info" {
            i = 1;
        }

        while i < tokens.len() - 1 {
            let key = tokens[i];
            if key == "pv" {
                result.variation = tokens[i + 1..].iter().map(|s| s.to_string()).collect();
                break;
            }
            i += 1;
            let value = tokens[i];
            match key {
                "order" => result.order = value.parse().unwrap_or(0),
                "move" => result.coordinate = value.to_string(),
                "visits" => result.playouts = value.parse().unwrap_or(0),
                "lcb" => result.lcb = value.parse::<f64>().unwrap_or(0.0) / 100.0,
                "prior" => result.policy = value.parse::<f64>().unwrap_or(0.0) / 100.0,
                "winrate" => result.winrate = value.parse::<f64>().unwrap_or(0.0) / 100.0,
                "scoreLead" => {
                    if is_sayuri {
                        result.score_mean = value.parse().unwrap_or(0.0);
                    }
                }
                "areas" => {
                    if !is_sayuri {
                        let areas: f64 = value.parse().unwrap_or(0.0);
                        result.score_mean = if black_to_play {
                            areas / 10000.0
                        } else {
                            -areas / 10000.0
                        };
                    }
                }
                _ => {}
            }
            i += 1;
        }

        result.is_kata_data = true;
        result.is_sai_data = true;
        Some(result)
    }

    /// Parse summary format: `Q4 -> 4348 (V: 43.88%) (LCB: 43.81%) (N: 18.67%) PV: Q4 D16`
    pub fn from_summary(summary: &str) -> Option<Self> {
        let s = summary.trim();
        if let Some(result) = parse_summary_with_lcb(s) {
            return Some(result);
        }
        parse_summary_old(s)
    }

    /// Parse KataGo summary: `Root info: 1000 visits, 50.5% win, -3.2 score, 22.1 stdev PV: D4 Q16`
    pub fn from_summary_kata(summary: &str) -> Option<Self> {
        let s = summary.trim();
        let s = s.strip_prefix("Root ").unwrap_or(s);
        let s = s.strip_prefix("info: ").unwrap_or(s);

        let s = if s.contains('=') {
            let parts: Vec<&str> = s.split('=').collect();
            if parts.len() >= 3 {
                format!("{}{}", parts[0], parts[1])
            } else {
                s.to_string()
            }
        } else {
            s.to_string()
        };

        let parts: Vec<&str> = s.split("PV").collect();
        if parts.is_empty() {
            return None;
        }

        let mut result = MoveData::default();
        let params: Vec<&str> = parts[0].trim().split_whitespace().collect();

        if params.len() >= 8 {
            result.is_kata_data = true;
            result.playouts = params[1].parse().unwrap_or(0);
            result.winrate = params[3].trim_end_matches('%').parse().unwrap_or(0.0);
            result.score_mean = params[5].parse().unwrap_or(0.0);
            result.score_stdev = params[7].parse().unwrap_or(0.0);
        }

        if parts.len() >= 2 {
            result.variation = parts[1].trim().split_whitespace().map(String::from).collect();
            result.coordinate = result.variation.first().cloned().unwrap_or_default();
        } else {
            result.coordinate = "A1".to_string();
            result.variation = vec!["A1".to_string()];
        }

        Some(result)
    }

    /// Parse Zen summary: `C15 -> 718, 47.87%, C15 O17 R14`
    pub fn from_summary_zen(summary: &str) -> Option<Self> {
        let params: Vec<&str> = summary.trim().split(',').collect();
        if params.len() < 3 {
            return None;
        }

        let mut result = MoveData::default();
        let parts: Vec<&str> = params[0].trim().split("->").collect();
        if parts.len() >= 2 {
            result.coordinate = parts[0].trim().to_string();
            result.playouts = parts[1].trim().parse().unwrap_or(0);
        }
        let wr = params[1].trim();
        result.winrate = wr.trim_end_matches('%').parse().unwrap_or(0.0);
        result.variation = params[2].trim().split_whitespace().map(String::from).collect();

        Some(result)
    }

    /// Calculate total playouts excluding symmetry duplicates.
    pub fn total_playouts(moves: &[MoveData]) -> usize {
        moves.iter().filter(|m| !m.is_symmetry).map(|m| m.playouts).sum()
    }
}

/// Parse summary with LCB: `Q4 -> 4348 (V: 43.88%) (LCB: 43.81%) (N: 18.67%) PV: Q4 D16`
fn parse_summary_with_lcb(s: &str) -> Option<MoveData> {
    let arrow_pos = s.find("->")?;
    let coord = s[..arrow_pos].trim().to_string();

    let pv_pos = s.find("PV:")?;
    let pv_str = s[pv_pos + 3..].trim();
    let variation: Vec<String> = pv_str.split_whitespace().map(String::from).collect();

    let middle = &s[arrow_pos + 2..pv_pos];
    let playouts: usize = middle.trim().split_whitespace().next()?.parse().ok()?;
    let winrate = extract_percent(middle, "V:")?;
    let lcb = extract_percent(middle, "LCB:").unwrap_or(winrate);

    Some(MoveData {
        coordinate: coord,
        playouts,
        winrate,
        lcb,
        variation,
        ..MoveData::default()
    })
}

/// Parse old format: `Q4 -> 4348 (V: 43.88%) (N: 18.67%) PV: Q4 D16`
fn parse_summary_old(s: &str) -> Option<MoveData> {
    let arrow_pos = s.find("->")?;
    let coord = s[..arrow_pos].trim().to_string();

    let pv_pos = s.find("PV:")?;
    let pv_str = s[pv_pos + 3..].trim();
    let variation: Vec<String> = pv_str.split_whitespace().map(String::from).collect();

    let middle = &s[arrow_pos + 2..pv_pos];
    let playouts: usize = middle.trim().split_whitespace().next()?.parse().ok()?;
    let winrate = extract_percent(middle, "V:")?;

    Some(MoveData {
        coordinate: coord,
        playouts,
        winrate,
        lcb: winrate,
        variation,
        ..MoveData::default()
    })
}

/// Extract a percentage from "(V: 43.88%)" or "(LCB: 43.81%)"
fn extract_percent(text: &str, prefix: &str) -> Option<f64> {
    let pos = text.find(prefix)?;
    let after = &text[pos + prefix.len()..];
    let trimmed = after.trim_start();
    let end = trimmed.find('%')?;
    trimmed[..end].trim().parse().ok()
}

impl fmt::Display for MoveData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} visits:{} wr:{:.1}% lcb:{:.1}% policy:{:.1}% score:{:.1}",
            self.coordinate, self.playouts, self.winrate, self.lcb, self.policy, self.score_mean
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_info_leela() {
        let line = "info move Q16 visits 80 winrate 4405 prior 1828 lcb 4379 order 0 pv Q16 D4";
        let result = MoveData::from_info(line).unwrap();
        assert_eq!(result.coordinate, "Q16");
        assert_eq!(result.playouts, 80);
        assert!((result.winrate - 44.05).abs() < 0.01);
        assert!((result.lcb - 43.79).abs() < 0.01);
        assert!((result.policy - 18.28).abs() < 0.01);
        assert_eq!(result.order, 0);
        assert_eq!(result.variation, vec!["Q16", "D4"]);
        assert!(!result.is_kata_data);
    }

    #[test]
    fn test_from_info_katago() {
        // KataGo actual format: scoreMean/scoreStdev BEFORE pv
        let line = "info move D4 visits 100 winrate 0.4405 prior 0.1828 lcb 0.4379 scoreMean -3.2 scoreStdev 22.1 order 0 pv D4 Q16";
        let result = MoveData::from_info_katago(line).unwrap();
        assert_eq!(result.coordinate, "D4");
        assert_eq!(result.playouts, 100);
        assert!((result.winrate - 44.05).abs() < 0.01);
        assert!((result.lcb - 43.79).abs() < 0.01);
        assert!((result.policy - 18.28).abs() < 0.01);
        assert!((result.score_mean - (-3.2)).abs() < 0.01);
        assert!((result.score_stdev - 22.1).abs() < 0.01);
        assert_eq!(result.variation, vec!["D4", "Q16"]);
        assert!(result.is_kata_data);
    }

    #[test]
    fn test_from_info_katago_with_pv_visits() {
        let line = "info move D4 visits 200 winrate 0.55 prior 0.3 lcb 0.54 order 0 pv D4 Q16 R6 pvVisits 100 80 20 movesOwnership 0.1 -0.2 0.3";
        let result = MoveData::from_info_katago(line).unwrap();
        assert_eq!(result.variation, vec!["D4", "Q16", "R6"]);
        assert_eq!(result.pv_visits, vec!["100", "80", "20"]);
        assert_eq!(result.moves_estimate_array, vec![0.1, -0.2, 0.3]);
    }

    #[test]
    fn test_from_info_katago_symmetry() {
        // isSymmetryOf appears before pv
        let line = "info move D4 visits 50 winrate 0.44 prior 0.1 lcb 0.43 isSymmetryOf E4 order 1 pv D4";
        let result = MoveData::from_info_katago(line).unwrap();
        assert!(result.is_symmetry);
    }

    #[test]
    fn test_from_info_sai_areas() {
        // areas appears before pv
        let line = "info move Q16 visits 50 winrate 4405 prior 1828 lcb 4379 areas 52000 order 0 pv Q16 D4";
        let result = MoveData::from_info_sai(line, false, true).unwrap();
        assert_eq!(result.coordinate, "Q16");
        assert!((result.score_mean - 5.2).abs() < 0.01);
        assert!(result.is_sai_data);
    }

    #[test]
    fn test_from_info_sai_sayuri() {
        // scoreLead appears before pv
        let line = "info move Q16 visits 50 winrate 4405 prior 1828 lcb 4379 scoreLead -3.5 order 0 pv Q16 D4";
        let result = MoveData::from_info_sai(line, true, true).unwrap();
        assert!((result.score_mean - (-3.5)).abs() < 0.01);
    }

    #[test]
    fn test_from_summary_with_lcb() {
        let summary = "Q4 -> 4348 (V: 43.88%) (LCB: 43.81%) (N: 18.67%) PV: Q4 D16 D4 Q16 R14";
        let result = MoveData::from_summary(summary).unwrap();
        assert_eq!(result.coordinate, "Q4");
        assert_eq!(result.playouts, 4348);
        assert!((result.winrate - 43.88).abs() < 0.01);
        assert!((result.lcb - 43.81).abs() < 0.01);
    }

    #[test]
    fn test_from_summary_old() {
        let summary = "Q4 -> 4348 (V: 43.88%) (N: 18.67%) PV: Q4 D16 D4 Q16";
        let result = MoveData::from_summary(summary).unwrap();
        assert_eq!(result.coordinate, "Q4");
        assert_eq!(result.playouts, 4348);
        assert!((result.winrate - 43.88).abs() < 0.01);
        assert!((result.lcb - 43.88).abs() < 0.01);
    }

    #[test]
    fn test_from_summary_zen() {
        let summary = "C15 -> 718, 47.87%, C15 O17 R14 Q18 R6";
        let result = MoveData::from_summary_zen(summary).unwrap();
        assert_eq!(result.coordinate, "C15");
        assert_eq!(result.playouts, 718);
        assert!((result.winrate - 47.87).abs() < 0.01);
    }

    #[test]
    fn test_total_playouts_excludes_symmetry() {
        let moves = vec![
            MoveData { playouts: 100, is_symmetry: false, ..MoveData::default() },
            MoveData { playouts: 50, is_symmetry: true, ..MoveData::default() },
            MoveData { playouts: 200, is_symmetry: false, ..MoveData::default() },
        ];
        assert_eq!(MoveData::total_playouts(&moves), 300);
    }
}
