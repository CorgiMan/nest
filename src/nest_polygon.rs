use std::f64::consts::{PI, TAU};

use geo::algorithm::orient::Direction;
use geo::algorithm::orient::Orient;
use geo::algorithm::translate::Translate;
use geo::{Coord, LineString, Polygon};

// to draw
use base64::{engine::general_purpose, Engine as _};

use image::{ImageBuffer, Rgba};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct NestPolygon {
    polygon: Polygon,
    slopes: Vec<f64>,
    is_convex: bool,
    zero_index: usize,
    pi_index: usize,
    minx: f64,
    maxx: f64,
    miny: f64,
    maxy: f64,
    bottom_left: Coord,
    // offset: Coord,
}

impl NestPolygon {
    // calculates properties of input polygon and returns a NestPolygon which stores them
    pub fn new(polygon: Vec<Coord>) -> NestPolygon {
        // todo must have more than 3 vertices
        let mut polygon = geo::Polygon::new(LineString::from(polygon), vec![]);
        polygon = polygon.orient(Direction::Default);
        let mut slopes = vec![];
        let points = polygon.exterior();

        for i in 0..points.0.len() - 1 {
            let d = points[i + 1] - points[i];
            slopes.push((d.y.atan2(d.x) + TAU) % TAU);
        }

        let mut zero_index = 0;
        let mut pi_index = 0;
        let mut prev = *slopes.last().unwrap();
        for (i, s) in slopes.iter().enumerate() {
            if prev > *s {
                zero_index = i;
            }
            if prev < PI && *s >= PI {
                pi_index = i;
            }
            prev = *s;
        }

        let mut is_convex = true;
        for i in 1..slopes.len() {
            let prev = (zero_index + i - 1) % slopes.len();
            let cur = (zero_index + i) % slopes.len();
            if slopes[prev] > slopes[cur] {
                is_convex = false;
            }
        }

        let mut minx = std::f64::MAX;
        let mut maxx = std::f64::MIN;
        let mut miny = std::f64::MAX;
        let mut maxy = std::f64::MIN;
        for Coord { x, y } in polygon.exterior().0.iter() {
            minx = if *x < minx { *x } else { minx };
            maxx = if *x > maxx { *x } else { maxx };
            miny = if *y < miny { *y } else { miny };
            maxy = if *y > maxy { *y } else { maxy };
        }

        let mut bestx = std::f64::MAX;
        for Coord { x, y } in polygon.exterior().0.iter() {
            if *y == miny {
                bestx = if *x < bestx { *x } else { bestx };
            }
        }
        let bottom_left = Coord { x: bestx, y: miny };

        NestPolygon {
            polygon,
            slopes,
            is_convex,
            zero_index,
            pi_index,
            minx,
            maxx,
            miny,
            maxy,
            bottom_left,
            // offset: Coord { x: 0., y: 0. },
        }
    }

    // draws polygons in iTerm
    fn draw(polygons: Vec<&NestPolygon>) {
        let mut minx = std::f64::MAX;
        let mut maxx = std::f64::MIN;
        let mut miny = std::f64::MAX;
        let mut maxy = std::f64::MIN;
        for p in polygons.iter() {
            minx = if p.minx < minx { p.minx } else { minx };
            maxx = if p.maxx > maxx { p.maxx } else { maxx };
            miny = if p.miny < miny { p.miny } else { miny };
            maxy = if p.maxy > maxy { p.maxy } else { maxy };
        }
        minx = minx - (maxx - minx) / 20.0;
        maxx = maxx + (maxx - minx) / 20.0;
        miny = miny - (maxy - miny) / 20.0;
        maxy = maxy + (maxy - miny) / 20.0;

        let dx = maxx - minx;
        let dy = maxy - miny;

        let w = 400.0;
        let h = f64::min(w / dx * dy, 10000.0);

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(w as u32, h as u32, Rgba([0, 0, 0, 255]));

        for p in polygons.iter() {
            let mut vertices: Vec<_> = p
                .polygon
                .exterior()
                .0
                .iter()
                .map(|p| imageproc::point::Point {
                    x: ((p.x - minx) * w / dx) as f32,
                    y: (h - (p.y - miny) * h / dy) as f32,
                })
                .collect();
            vertices.pop();

            let mut start = vertices.last().unwrap();
            for end in vertices.iter() {
                imageproc::drawing::draw_line_segment_mut(
                    &mut img,
                    (start.x, start.y),
                    (end.x, end.y),
                    Rgba([0, 0, 255, 188]),
                );
                start = end;
            }
        }

        // // imageproc::drawing::draw_polygon_mut(&mut img, &vertices, Rgba([0, 0, 255, 188]));

        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageOutputFormat::Png)
            .unwrap();

        let b64 = general_purpose::STANDARD_NO_PAD.encode(buffer.get_ref());
        print!("\x1B]1337;File=inline=1;size={}:", b64.len());
        println!("{}", b64);
        println!("\x07");
    }

    // does NFP for convex polygons
    fn minkowski_sum(&self, other: &NestPolygon) -> NestPolygon {
        if !self.is_convex {
            // todo: concave version
            panic!("can't get NFP for concave polygon")
        };

        let mut current_loc = self.bottom_left;
        let mut i1 = self.zero_index;
        let mut i2 = other.pi_index;
        let mut vec = Vec::<Coord>::new();
        vec.push(current_loc);
        loop {
            let s1 = self.slopes[i1];
            let s2 = (other.slopes[i2] + PI) % TAU;

            if s1 <= s2 {
                current_loc =
                    current_loc + self.polygon.exterior().0[i1 + 1] - self.polygon.exterior().0[i1];
                vec.push(current_loc);
                i1 = (i1 + 1) % self.slopes.len();
                if i1 == self.zero_index {
                    loop {
                        current_loc = current_loc + other.polygon.exterior().0[i2]
                            - other.polygon.exterior().0[i2 + 1];
                        vec.push(current_loc);
                        i2 = (i2 + 1) % other.slopes.len();
                        if i2 == other.pi_index {
                            break;
                        }
                    }
                    break;
                }
            } else {
                current_loc = current_loc + other.polygon.exterior().0[i2]
                    - other.polygon.exterior().0[i2 + 1];
                vec.push(current_loc);
                i2 = (i2 + 1) % other.slopes.len();
                if i2 == other.pi_index {
                    loop {
                        current_loc = current_loc + self.polygon.exterior().0[i1 + 1]
                            - self.polygon.exterior().0[i1];
                        vec.push(current_loc);
                        i1 = (i1 + 1) % self.slopes.len();
                        if i1 == self.zero_index {
                            break;
                        }
                    }
                    break;
                }
            }
        }
        NestPolygon::new(vec)
    }
}

pub fn f() {
    let _p1 = NestPolygon::new(vec![
        Coord { x: 20., y: 20. },
        Coord { x: 20., y: 40. },
        Coord { x: 40., y: 40. },
        Coord { x: 40., y: 20. },
    ]);

    let _p2 = NestPolygon::new(vec![
        Coord { x: 20., y: 20. },
        Coord { x: 30., y: 30. },
        Coord { x: 20., y: 40. },
        Coord { x: 10., y: 30. },
    ]);

    let mut _p3 = NestPolygon::new(vec![
        Coord { x: 20.0, y: 10.0 },
        Coord { x: 30.0, y: 0.0 },
        Coord { x: 40.0, y: 0.0 },
        Coord { x: 50.0, y: 10.0 },
        Coord { x: 60.0, y: 20.0 },
        Coord { x: 70.0, y: 30.0 },
        Coord { x: 50.0, y: 70.0 },
    ]);

    let mut _p4 = NestPolygon::new(vec![
        Coord { x: 0.0, y: 20.0 },
        Coord { x: 10.0, y: 40.0 },
        Coord { x: 20.0, y: 50.0 },
        Coord { x: 40.0, y: 60.0 },
        Coord { x: 60.0, y: 30.0 },
        Coord { x: 40.0, y: 0.0 },
        Coord { x: 20.0, y: 10.0 },
    ]);

    let _p5 = NestPolygon::new(vec![
        Coord { x: 70.0, y: 10.0 },
        Coord { x: 80.0, y: 20.0 },
        Coord { x: 90.0, y: 40.0 },
        Coord { x: 80.0, y: 60.0 },
        Coord { x: 60.0, y: 70.0 },
        Coord { x: 50.0, y: 50.0 },
        Coord { x: 60.0, y: 30.0 },
    ]);

    let nfp = _p3.minkowski_sum(&_p4);
    crate::p!(nfp.polygon.exterior().0.last().unwrap());
    let trans = _p3.polygon.exterior().0[_p3.zero_index] - _p4.polygon.exterior().0[_p4.pi_index];
    _p4.polygon.translate_mut(trans.x, trans.y);
    // let first = nfp.polygon.exterior().0.first().unwrap();

    for i in 1..nfp.polygon.exterior().0.len() {
        let d = nfp.polygon.exterior().0[i] - nfp.polygon.exterior().0[i - 1];
        _p4.polygon.translate_mut(d.x, d.y);
        NestPolygon::draw(vec![&_p3, &_p4, &nfp]);
    }

    // _p2.minkowski_sum(&_p3).draw();
}
