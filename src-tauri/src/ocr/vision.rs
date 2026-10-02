//! The picture arithmetic the OCR engine is built out of: resizing, the
//! geometry that turns the detector's heat map into text lines, and cutting
//! those lines out of the screenshot for the recogniser.
//!
//! Everything here mirrors what RapidOCR does with OpenCV and NumPy, down to
//! the convention that a pixel is sampled at its centre — `dst = (src + 0.5) *
//! scale - 0.5` — because the models were trained on images resized that way
//! and a different convention costs accuracy. Two things are filled in with
//! plain `f32` code instead of OpenCV: the cut-out of a line is an affine step
//! across the quad rather than a full perspective transform, and a line is
//! resampled bilinearly rather than bicubically. Both are inside the tolerance
//! the recogniser was trained with, and text detection boxes are close to
//! parallelograms to begin with.

/// A point in an image, in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

/// An image with three channels in the order the models want them: blue,
/// green, red. Screenshots arrive in that order inside their alpha-bearing
/// pixels, so nothing is swapped on the way in.
#[derive(Debug, Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    /// `width * height * 3` bytes, row by row.
    pub data: Vec<u8>,
}

impl Image {
    /// Drops the alpha channel of a screenshot.
    pub fn from_bgra(pixels: &[u8], width: usize, height: usize) -> Image {
        let mut data = Vec::with_capacity(width * height * 3);
        for pixel in pixels.chunks_exact(4).take(width * height) {
            data.extend_from_slice(&pixel[..3]);
        }
        // A screenshot that claims more pixels than it carries is padded black
        // rather than read out of bounds.
        data.resize(width * height * 3, 0);
        Image {
            width,
            height,
            data,
        }
    }

    fn at(&self, x: usize, y: usize) -> [u8; 3] {
        let start = (y * self.width + x) * 3;
        [self.data[start], self.data[start + 1], self.data[start + 2]]
    }

    /// The colour at a point, with points outside the image taking the colour
    /// of the nearest edge pixel, as `BORDER_REPLICATE` does.
    fn sample(&self, x: f32, y: f32) -> [f32; 3] {
        let last_x = self.width.saturating_sub(1) as f32;
        let last_y = self.height.saturating_sub(1) as f32;
        let x = x.clamp(0.0, last_x);
        let y = y.clamp(0.0, last_y);
        let x0 = x.floor();
        let y0 = y.floor();
        let x1 = (x0 + 1.0).min(last_x);
        let y1 = (y0 + 1.0).min(last_y);
        let fx = x - x0;
        let fy = y - y0;
        let (x0, y0, x1, y1) = (x0 as usize, y0 as usize, x1 as usize, y1 as usize);
        let mut out = [0f32; 3];
        for (channel, value) in out.iter_mut().enumerate() {
            let top =
                self.at(x0, y0)[channel] as f32 * (1.0 - fx) + self.at(x1, y0)[channel] as f32 * fx;
            let bottom =
                self.at(x0, y1)[channel] as f32 * (1.0 - fx) + self.at(x1, y1)[channel] as f32 * fx;
            *value = top * (1.0 - fy) + bottom * fy;
        }
        out
    }

    /// Resizes to an exact size, bilinearly, sampling at pixel centres.
    pub fn resize(&self, width: usize, height: usize) -> Image {
        let mut data = Vec::with_capacity(width * height * 3);
        if width == 0 || height == 0 {
            return Image {
                width: 0,
                height: 0,
                data,
            };
        }
        let scale_x = self.width as f32 / width as f32;
        let scale_y = self.height as f32 / height as f32;
        for y in 0..height {
            let source_y = (y as f32 + 0.5) * scale_y - 0.5;
            for x in 0..width {
                let source_x = (x as f32 + 0.5) * scale_x - 0.5;
                for value in self.sample(source_x, source_y) {
                    data.push(value.round().clamp(0.0, 255.0) as u8);
                }
            }
        }
        Image {
            width,
            height,
            data,
        }
    }

    /// The image as the model input tensor wants it: one channel after another
    /// rather than colour by colour, scaled to a mean of zero and a spread of
    /// one.
    pub fn normalized(&self) -> Vec<f32> {
        let plane = self.width * self.height;
        let mut out = vec![0f32; plane * 3];
        for (index, byte) in self.data.iter().enumerate() {
            let channel = index % 3;
            let pixel = index / 3;
            out[channel * plane + pixel] = (*byte as f32 / 255.0 - 0.5) / 0.5;
        }
        out
    }

    /// The image rotated a quarter turn anticlockwise, as `numpy.rot90` does.
    pub fn rot90(&self) -> Image {
        let mut data = vec![0u8; self.data.len()];
        for y in 0..self.height {
            for x in 0..self.width {
                let target = ((self.width - 1 - x) * self.height + y) * 3;
                let source = (y * self.width + x) * 3;
                data[target..target + 3].copy_from_slice(&self.data[source..source + 3]);
            }
        }
        Image {
            width: self.height,
            height: self.width,
            data,
        }
    }

    /// Cuts a text line out of the image.
    ///
    /// `quad` runs top-left, top-right, bottom-right, bottom-left, the order the
    /// detector hands back. A line that came out taller than it is wide is
    /// turned on its side, because the recogniser only reads horizontally.
    pub fn crop(&self, quad: &[Pt; 4]) -> Option<Image> {
        let width = length(quad[0], quad[1]).max(length(quad[2], quad[3])) as usize;
        let height = length(quad[0], quad[3]).max(length(quad[1], quad[2])) as usize;
        if width < 1 || height < 1 {
            return None;
        }

        // The quad is walked by a pair of fractions running over its two axes,
        // which maps the rectangle onto it without a matrix solve.
        let mut data = Vec::with_capacity(width * height * 3);
        for y in 0..height {
            let v = (y as f32 + 0.5) / height as f32;
            for x in 0..width {
                let u = (x as f32 + 0.5) / width as f32;
                let top = lerp(quad[0], quad[1], u);
                let bottom = lerp(quad[3], quad[2], u);
                let point = lerp(top, bottom, v);
                for value in self.sample(point.x, point.y) {
                    data.push(value.round().clamp(0.0, 255.0) as u8);
                }
            }
        }
        let cropped = Image {
            width,
            height,
            data,
        };
        if height as f32 / width as f32 >= 1.5 {
            Some(cropped.rot90())
        } else {
            Some(cropped)
        }
    }
}

fn lerp(from: Pt, to: Pt, t: f32) -> Pt {
    Pt {
        x: from.x + (to.x - from.x) * t,
        y: from.y + (to.y - from.y) * t,
    }
}

fn length(from: Pt, to: Pt) -> f32 {
    ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt()
}

/// A rectangle that holds a set of points, with the points in the detector's
/// order.
#[derive(Debug, Clone)]
pub struct Rect {
    pub points: [Pt; 4],
    /// The shorter of the rectangle's two sides.
    pub side: f32,
}

impl Rect {
    pub fn area(&self) -> f32 {
        let [a, b, c, d] = self.points;
        // The shoelace formula, which the detector's unclipping step uses to
        // decide how far out to push a box.
        0.5 * ((a.x * b.y - b.x * a.y)
            + (b.x * c.y - c.x * b.y)
            + (c.x * d.y - d.x * c.y)
            + (d.x * a.y - a.x * d.y))
            .abs()
    }

    pub fn perimeter(&self) -> f32 {
        let [a, b, c, d] = self.points;
        length(a, b) + length(b, c) + length(c, d) + length(d, a)
    }

    /// The rectangle grown by `distance` on every side. The detector's
    /// unclipping step pushes a box outwards with rounded joins and then takes
    /// the smallest rectangle around the result, which for a rectangle is the
    /// rectangle grown by that distance.
    pub fn grown(&self, distance: f32) -> Rect {
        let edge = |from: Pt, to: Pt| {
            let dx = to.x - from.x;
            let dy = to.y - from.y;
            let len = (dx * dx + dy * dy).sqrt();
            (
                from,
                Pt {
                    x: dx / len,
                    y: dy / len,
                },
            )
        };
        let outward = |from: Pt, to: Pt, direction: Pt| {
            // The normal that points away from the centre of the rectangle.
            let normal = Pt {
                x: -direction.y,
                y: direction.x,
            };
            let middle = Pt {
                x: (from.x + to.x) / 2.0,
                y: (from.y + to.y) / 2.0,
            };
            let away = (middle.x - (self.points[0].x + self.points[2].x) / 2.0) * normal.x
                + (middle.y - (self.points[0].y + self.points[2].y) / 2.0) * normal.y;
            if away < 0.0 {
                Pt {
                    x: -normal.x,
                    y: -normal.y,
                }
            } else {
                normal
            }
        };

        let mut shifted = [(Pt { x: 0.0, y: 0.0 }, Pt { x: 0.0, y: 0.0 }); 4];
        for (index, slot) in shifted.iter_mut().enumerate() {
            let (from, direction) = edge(self.points[index], self.points[(index + 1) % 4]);
            let normal = outward(from, self.points[(index + 1) % 4], direction);
            *slot = (
                Pt {
                    x: from.x + normal.x * distance,
                    y: from.y + normal.y * distance,
                },
                direction,
            );
        }

        // A corner is where the two edges that meet there cross, now that both
        // have been pushed out.
        let out = std::array::from_fn(|index| {
            let (from, direction) = shifted[(index + 3) % 4];
            let (other_from, other_direction) = shifted[index];
            let denominator = direction.x * other_direction.y - direction.y * other_direction.x;
            if denominator.abs() <= f32::EPSILON {
                return self.points[index];
            }
            let gap = Pt {
                x: other_from.x - from.x,
                y: other_from.y - from.y,
            };
            let along = (gap.x * other_direction.y - gap.y * other_direction.x) / denominator;
            Pt {
                x: from.x + direction.x * along,
                y: from.y + direction.y * along,
            }
        });
        Rect {
            points: out,
            side: self.side + distance * 2.0,
        }
    }
}

/// The smallest rectangle that encloses the points, found by trying every
/// direction of the convex hull's edges. The points come back the way the
/// detector orders them: top-left, top-right, bottom-right, bottom-left.
pub fn min_area_rect(points: &[Pt]) -> Rect {
    let hull = hull(points);
    if hull.len() < 3 {
        return bounding_rect(points);
    }

    let mut best: Option<(f32, [Pt; 4], f32)> = None;
    for index in 0..hull.len() {
        let from = hull[index];
        let to = hull[(index + 1) % hull.len()];
        let edge = Pt {
            x: to.x - from.x,
            y: to.y - from.y,
        };
        let len = (edge.x * edge.x + edge.y * edge.y).sqrt();
        if len <= f32::EPSILON {
            continue;
        }
        let axis = Pt {
            x: edge.x / len,
            y: edge.y / len,
        };
        let normal = Pt {
            x: -axis.y,
            y: axis.x,
        };
        let (mut min_u, mut max_u, mut min_v, mut max_v) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for point in &hull {
            let u = point.x * axis.x + point.y * axis.y;
            let v = point.x * normal.x + point.y * normal.y;
            min_u = min_u.min(u);
            max_u = max_u.max(u);
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }
        let area = (max_u - min_u) * (max_v - min_v);
        if best.map(|(smallest, _, _)| area < smallest).unwrap_or(true) {
            let corner = |u: f32, v: f32| Pt {
                x: axis.x * u + normal.x * v,
                y: axis.y * u + normal.y * v,
            };
            let quad = [
                corner(min_u, min_v),
                corner(max_u, min_v),
                corner(max_u, max_v),
                corner(min_u, max_v),
            ];
            best = Some((area, quad, (max_v - min_v).min(max_u - min_u)));
        }
    }

    let Some((_, quad, side)) = best else {
        return bounding_rect(points);
    };
    Rect {
        points: order(quad),
        side,
    }
}

/// Puts a rectangle's corners in the detector's order: top-left, top-right,
/// bottom-right, bottom-left.
fn order(quad: [Pt; 4]) -> [Pt; 4] {
    let mut sorted = quad;
    sorted.sort_by(|left, right| {
        left.x
            .partial_cmp(&right.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    // Of the two corners on the left, the higher one is the top-left; of the
    // two on the right, the higher one is the top-right.
    let (top_left, bottom_left) = if sorted[1].y > sorted[0].y {
        (0, 1)
    } else {
        (1, 0)
    };
    let (top_right, bottom_right) = if sorted[3].y > sorted[2].y {
        (2, 3)
    } else {
        (3, 2)
    };
    [
        sorted[top_left],
        sorted[top_right],
        sorted[bottom_right],
        sorted[bottom_left],
    ]
}

/// The axis-aligned rectangle around the points, for point sets too small to
/// have a shape of their own.
fn bounding_rect(points: &[Pt]) -> Rect {
    if points.is_empty() {
        return Rect {
            points: [Pt { x: 0.0, y: 0.0 }; 4],
            side: 0.0,
        };
    }
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for point in points {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }
    Rect {
        points: [
            Pt { x: min_x, y: min_y },
            Pt { x: max_x, y: min_y },
            Pt { x: max_x, y: max_y },
            Pt { x: min_x, y: max_y },
        ],
        side: (max_x - min_x).min(max_y - min_y),
    }
}

/// The outline of a set of points, without the points that lie inside it.
///
/// Andrew's monotone chain, the same outline `cv2.convexHull` produces.
pub fn hull(points: &[Pt]) -> Vec<Pt> {
    let mut sorted: Vec<Pt> = points.to_vec();
    sorted.sort_by(|left, right| {
        left.x
            .partial_cmp(&right.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                left.y
                    .partial_cmp(&right.y)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    sorted.dedup_by(|left, right| left.x == right.x && left.y == right.y);
    if sorted.len() < 3 {
        return sorted;
    }

    let turn = |origin: Pt, a: Pt, b: Pt| {
        (a.x - origin.x) * (b.y - origin.y) - (a.y - origin.y) * (b.x - origin.x)
    };
    let mut lower: Vec<Pt> = Vec::with_capacity(sorted.len());
    for point in &sorted {
        while lower.len() >= 2
            && turn(lower[lower.len() - 2], lower[lower.len() - 1], *point) <= 0.0
        {
            lower.pop();
        }
        lower.push(*point);
    }
    let mut upper: Vec<Pt> = Vec::with_capacity(sorted.len());
    for point in sorted.iter().rev() {
        while upper.len() >= 2
            && turn(upper[upper.len() - 2], upper[upper.len() - 1], *point) <= 0.0
        {
            upper.pop();
        }
        upper.push(*point);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Whether the image has ink at a pixel, in the squared-up form the detector's
/// heat map is read in. Coordinates are whole pixels, the way OpenCV truncates
/// them when it fills a shape.
pub fn quad_contains(quad: &[Pt; 4], x: i32, y: i32) -> bool {
    let point = Pt {
        x: x as f32 + 0.5,
        y: y as f32 + 0.5,
    };
    let mut inside = false;
    let mut previous = 3;
    for current in 0..4 {
        let a = quad[previous];
        let b = quad[current];
        if (a.y > point.y) != (b.y > point.y)
            && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

/// Grows a mask by one pixel in every direction, the way the detector dilates
/// its heat map before looking for text lines. A 2x2 kernel anchored top-left:
/// each pixel takes the largest of itself and its right, lower and lower-right
/// neighbours.
pub fn dilate(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    let mut out = vec![false; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut on = mask[y * width + x];
            for (dx, dy) in [(1, 0), (0, 1), (1, 1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < width && ny < height {
                    on = on || mask[ny * width + nx];
                }
            }
            out[y * width + x] = on;
        }
    }
    out
}

/// Groups the pixels that are on into shapes, eight-connected the way OpenCV
/// traces contours. `limit` stops the search early on a very noisy picture,
/// matching the cap the detector puts on how many boxes it will consider.
pub fn components(mask: &[bool], width: usize, height: usize, limit: usize) -> Vec<Vec<Pt>> {
    let mut seen = vec![false; mask.len()];
    let mut shapes = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || seen[start] {
            continue;
        }
        if shapes.len() >= limit {
            break;
        }
        let mut shape = Vec::new();
        seen[start] = true;
        stack.push(start);
        while let Some(index) = stack.pop() {
            let (x, y) = (index % width, index / width);
            shape.push(Pt {
                x: x as f32,
                y: y as f32,
            });
            for (dx, dy) in [
                (-1i32, -1i32),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                    continue;
                }
                let neighbour = ny as usize * width + nx as usize;
                if mask[neighbour] && !seen[neighbour] {
                    seen[neighbour] = true;
                    stack.push(neighbour);
                }
            }
        }
        shapes.push(shape);
    }
    shapes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(width: usize, height: usize, value: u8) -> Image {
        Image {
            width,
            height,
            data: vec![value; width * height * 3],
        }
    }

    #[test]
    fn alpha_is_dropped_and_the_channels_keep_their_order() {
        let bgra = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let image = Image::from_bgra(&bgra, 2, 1);
        assert_eq!(image.data, vec![1, 2, 3, 5, 6, 7]);
    }

    #[test]
    fn a_screenshot_that_is_short_of_pixels_is_padded_rather_than_read_past() {
        // A 2x2 screenshot carrying one pixel: the rest is read as black.
        let image = Image::from_bgra(&[9, 9, 9, 9], 2, 2);
        assert_eq!(image.data, vec![9, 9, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn a_resize_to_the_same_size_changes_nothing() {
        let mut source = image(4, 3, 0);
        for (index, byte) in source.data.iter_mut().enumerate() {
            *byte = (index * 7 % 251) as u8;
        }
        assert_eq!(source.resize(4, 3).data, source.data);
    }

    #[test]
    fn a_resize_averages_between_the_pixels_it_lands_between() {
        let mut source = image(2, 1, 0);
        source.data = vec![0, 0, 0, 200, 200, 200];
        // Sampling at pixel centres of a one-pixel-wide result lands halfway
        // between the two source pixels.
        let resized = source.resize(1, 1);
        assert_eq!(resized.data, vec![100, 100, 100]);
    }

    #[test]
    fn normalising_puts_the_colours_on_a_scale_of_minus_one_to_one() {
        let mut source = image(2, 1, 0);
        source.data = vec![0, 0, 0, 255, 255, 255];
        // Channel by channel, not colour by colour.
        assert_eq!(source.normalized(), vec![-1.0, 1.0, -1.0, 1.0, -1.0, 1.0]);
    }

    #[test]
    fn normalising_a_second_colour_keeps_the_channels_apart() {
        let mut source = image(2, 1, 0);
        source.data = vec![0, 255, 0, 0, 0, 255];
        assert_eq!(source.normalized(), vec![-1.0, -1.0, 1.0, -1.0, -1.0, 1.0]);
    }

    #[test]
    fn a_quarter_turn_swaps_the_sides_and_keeps_the_pixels() {
        let mut source = image(2, 1, 0);
        source.data = vec![1, 1, 1, 2, 2, 2];
        let turned = source.rot90();
        assert_eq!((turned.width, turned.height), (1, 2));
        assert_eq!(turned.data, vec![2, 2, 2, 1, 1, 1]);
    }

    #[test]
    fn a_line_is_cut_out_along_its_own_box() {
        let source = image(4, 4, 255);
        let quad = [
            Pt { x: 0.0, y: 0.0 },
            Pt { x: 2.0, y: 0.0 },
            Pt { x: 2.0, y: 4.0 },
            Pt { x: 0.0, y: 4.0 },
        ];
        let cropped = source.crop(&quad).unwrap();
        // Tall and narrow: it ends up turned on its side.
        assert_eq!((cropped.width, cropped.height), (4, 2));
        assert!(cropped.data.iter().all(|byte| *byte == 255));
    }

    #[test]
    fn a_line_that_is_already_wide_is_left_alone() {
        // The top half of the image is white. A box over it, inset so that
        // every sample lands on a whole pixel, comes back white and the right
        // way up.
        let mut source = image(6, 6, 0);
        for y in 0..3 {
            for x in 0..6 {
                let start = (y * 6 + x) * 3;
                source.data[start..start + 3].copy_from_slice(&[255, 255, 255]);
            }
        }
        let quad = [
            Pt { x: 0.5, y: 0.5 },
            Pt { x: 4.5, y: 0.5 },
            Pt { x: 4.5, y: 2.5 },
            Pt { x: 0.5, y: 2.5 },
        ];
        let cropped = source.crop(&quad).unwrap();
        assert_eq!((cropped.width, cropped.height), (4, 2));
        assert!(cropped.data.iter().all(|byte| *byte == 255));
    }

    #[test]
    fn a_box_with_no_size_is_not_cut_out() {
        let source = image(4, 4, 0);
        let quad = [Pt { x: 1.0, y: 1.0 }; 4];
        assert!(source.crop(&quad).is_none());
    }

    #[test]
    fn the_smallest_rectangle_around_a_square_is_that_square() {
        let mut points = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                points.push(Pt {
                    x: x as f32,
                    y: y as f32,
                });
            }
        }
        let rect = min_area_rect(&points);
        assert!((rect.area() - 81.0).abs() < 0.5, "{:?}", rect);
        assert!((rect.side - 9.0).abs() < 0.5, "{:?}", rect);
        assert_eq!(rect.points[0], Pt { x: 0.0, y: 0.0 });
    }

    #[test]
    fn the_smallest_rectangle_of_a_tilted_line_follows_its_tilt() {
        // A 20 by 4 line, turned by 45 degrees about the origin.
        let angle = std::f32::consts::FRAC_PI_4;
        let corners = [(0.0, 0.0), (20.0, 0.0), (20.0, 4.0), (0.0, 4.0)];
        let points: Vec<Pt> = corners
            .iter()
            .map(|(x, y)| Pt {
                x: x * angle.cos() - y * angle.sin(),
                y: x * angle.sin() + y * angle.cos(),
            })
            .collect();
        let rect = min_area_rect(&points);
        assert!((rect.area() - 80.0).abs() < 1.0, "{:?}", rect);
        assert!((rect.side - 4.0).abs() < 0.5, "{:?}", rect);
        // The corners still run around the rectangle in the detector's order.
        let edges: Vec<f32> = (0..4)
            .map(|index| length(rect.points[index], rect.points[(index + 1) % 4]))
            .collect();
        assert!((edges[0] - 20.0).abs() < 1.0, "{edges:?}");
        assert!((edges[1] - 4.0).abs() < 1.0, "{edges:?}");
    }

    #[test]
    fn a_rectangle_grows_by_the_distance_on_every_side() {
        let rect = Rect {
            points: [
                Pt { x: 0.0, y: 0.0 },
                Pt { x: 10.0, y: 0.0 },
                Pt { x: 10.0, y: 4.0 },
                Pt { x: 0.0, y: 4.0 },
            ],
            side: 4.0,
        };
        let grown = rect.grown(2.0);
        assert!((grown.area() - (14.0 * 8.0)).abs() < 0.01, "{:?}", grown);
        assert!((grown.perimeter() - (14.0 * 2.0 + 8.0 * 2.0)).abs() < 0.01);
        assert!((grown.side - 8.0).abs() < 0.01);
        assert!((grown.points[0].x + 2.0).abs() < 0.01, "{:?}", grown);
        assert!((grown.points[0].y + 2.0).abs() < 0.01, "{:?}", grown);
    }

    #[test]
    fn the_outline_of_a_square_drops_the_points_inside_it() {
        let mut points = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                points.push(Pt {
                    x: x as f32,
                    y: y as f32,
                });
            }
        }
        let outline = hull(&points);
        assert_eq!(outline.len(), 4);
        assert!(outline.contains(&Pt { x: 0.0, y: 0.0 }));
        assert!(outline.contains(&Pt { x: 4.0, y: 4.0 }));
    }

    #[test]
    fn a_point_inside_a_rectangle_is_inside_it() {
        let quad = [
            Pt { x: 0.0, y: 0.0 },
            Pt { x: 8.0, y: 0.0 },
            Pt { x: 8.0, y: 4.0 },
            Pt { x: 0.0, y: 4.0 },
        ];
        assert!(quad_contains(&quad, 4, 2));
        assert!(!quad_contains(&quad, 9, 2));
        assert!(!quad_contains(&quad, 4, 5));
    }

    #[test]
    fn growing_a_mask_takes_in_the_pixels_above_and_to_the_left() {
        // The kernel is anchored at its top-left corner, so a pixel that is on
        // lights up the block that ends at it, not the one that starts at it.
        let mask = [false, true, false, false];
        assert_eq!(dilate(&mask, 2, 2), vec![true, true, false, false]);
    }

    #[test]
    fn pixels_that_touch_at_a_corner_are_one_shape() {
        let mask = [true, false, false, true];
        let shapes = components(&mask, 2, 2, 100);
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].len(), 2);

        let separated = [true, false, false, false, false, false, false, false, true];
        assert_eq!(components(&separated, 3, 3, 100).len(), 2);
    }

    #[test]
    fn the_shapes_found_stop_at_the_limit() {
        let mask = [true, false, true];
        assert_eq!(components(&mask, 3, 1, 2).len(), 2);
        assert_eq!(components(&mask, 3, 1, 1).len(), 1);
    }
}
