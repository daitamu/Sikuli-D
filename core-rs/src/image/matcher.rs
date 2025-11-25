//! Template matching implementation

use crate::{Match, Pattern, Region, Result};
use image::{DynamicImage, GrayImage};

/// Image matcher for template matching
pub struct ImageMatcher {
    /// Minimum similarity threshold
    min_similarity: f64,
}

impl Default for ImageMatcher {
    fn default() -> Self {
        Self {
            min_similarity: 0.7,
        }
    }
}

impl ImageMatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum similarity threshold
    pub fn with_min_similarity(mut self, similarity: f64) -> Self {
        self.min_similarity = similarity.clamp(0.0, 1.0);
        self
    }

    /// Find a pattern in the screen image
    ///
    /// Uses normalized cross-correlation for template matching
    pub fn find(
        &self,
        screen: &DynamicImage,
        pattern: &Pattern,
    ) -> Result<Option<Match>> {
        let template = super::load_image_from_bytes(&pattern.image_data)?;

        let screen_gray = screen.to_luma8();
        let template_gray = template.to_luma8();

        let (sw, sh) = screen_gray.dimensions();
        let (tw, th) = template_gray.dimensions();

        if tw > sw || th > sh {
            return Ok(None);
        }

        let mut best_score = 0.0f64;
        let mut best_pos = (0u32, 0u32);

        // Sliding window template matching
        for y in 0..=(sh - th) {
            for x in 0..=(sw - tw) {
                let score = self.calculate_ncc(&screen_gray, &template_gray, x, y);
                if score > best_score {
                    best_score = score;
                    best_pos = (x, y);
                }
            }
        }

        let threshold = pattern.similarity.max(self.min_similarity);

        if best_score >= threshold {
            let region = Region::new(
                best_pos.0 as i32,
                best_pos.1 as i32,
                tw,
                th,
            );
            Ok(Some(Match::new(region, best_score)))
        } else {
            Ok(None)
        }
    }

    /// Find all occurrences of a pattern
    pub fn find_all(
        &self,
        screen: &DynamicImage,
        pattern: &Pattern,
    ) -> Result<Vec<Match>> {
        let template = super::load_image_from_bytes(&pattern.image_data)?;

        let screen_gray = screen.to_luma8();
        let template_gray = template.to_luma8();

        let (sw, sh) = screen_gray.dimensions();
        let (tw, th) = template_gray.dimensions();

        if tw > sw || th > sh {
            return Ok(vec![]);
        }

        let threshold = pattern.similarity.max(self.min_similarity);
        let mut matches = Vec::new();

        // Sliding window with non-maximum suppression
        for y in 0..=(sh - th) {
            for x in 0..=(sw - tw) {
                let score = self.calculate_ncc(&screen_gray, &template_gray, x, y);
                if score >= threshold {
                    let region = Region::new(x as i32, y as i32, tw, th);
                    matches.push(Match::new(region, score));
                }
            }
        }

        // Non-maximum suppression
        matches = self.non_maximum_suppression(matches, tw, th);

        Ok(matches)
    }

    /// Calculate Normalized Cross-Correlation (NCC)
    fn calculate_ncc(
        &self,
        screen: &GrayImage,
        template: &GrayImage,
        offset_x: u32,
        offset_y: u32,
    ) -> f64 {
        let (tw, th) = template.dimensions();

        let mut sum_st = 0.0f64;
        let mut sum_s2 = 0.0f64;
        let mut sum_t2 = 0.0f64;

        for ty in 0..th {
            for tx in 0..tw {
                let s = screen.get_pixel(offset_x + tx, offset_y + ty)[0] as f64;
                let t = template.get_pixel(tx, ty)[0] as f64;

                sum_st += s * t;
                sum_s2 += s * s;
                sum_t2 += t * t;
            }
        }

        let denominator = (sum_s2 * sum_t2).sqrt();
        if denominator < f64::EPSILON {
            0.0
        } else {
            sum_st / denominator
        }
    }

    /// Non-maximum suppression to remove overlapping matches
    fn non_maximum_suppression(
        &self,
        mut matches: Vec<Match>,
        _template_width: u32,
        _template_height: u32,
    ) -> Vec<Match> {
        // Sort by score descending
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        let mut result = Vec::new();
        let mut suppressed = vec![false; matches.len()];

        for i in 0..matches.len() {
            if suppressed[i] {
                continue;
            }

            result.push(matches[i].clone());

            // Suppress overlapping matches
            for j in (i + 1)..matches.len() {
                if suppressed[j] {
                    continue;
                }

                let overlap = self.calculate_overlap(
                    &matches[i].region,
                    &matches[j].region,
                );

                // Suppress if overlap > 50%
                if overlap > 0.5 {
                    suppressed[j] = true;
                }
            }
        }

        result
    }

    /// Calculate overlap ratio between two regions
    fn calculate_overlap(&self, a: &Region, b: &Region) -> f64 {
        let x1 = a.x.max(b.x);
        let y1 = a.y.max(b.y);
        let x2 = (a.x + a.width as i32).min(b.x + b.width as i32);
        let y2 = (a.y + a.height as i32).min(b.y + b.height as i32);

        if x1 >= x2 || y1 >= y2 {
            return 0.0;
        }

        let intersection = ((x2 - x1) * (y2 - y1)) as f64;
        let area_a = (a.width * a.height) as f64;
        let area_b = (b.width * b.height) as f64;
        let union = area_a + area_b - intersection;

        intersection / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matcher_creation() {
        let matcher = ImageMatcher::new().with_min_similarity(0.8);
        assert!((matcher.min_similarity - 0.8).abs() < f64::EPSILON);
    }
}
