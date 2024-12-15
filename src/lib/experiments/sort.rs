use crate::{anim, shape, Referer, Slide};

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h = h % 360.0;
    let s = s.max(0.0).min(100.0) / 100.0;
    let l = l.max(0.0).min(100.0) / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r + m) * 255.0).round() as u8;
    let g = ((g + m) * 255.0).round() as u8;
    let b = ((b + m) * 255.0).round() as u8;

    (r, g, b)
}

fn rainbow(i: usize, m: usize, s: f32, l: f32) -> (u8, u8, u8) {
    hsl_to_rgb(i as f32 * 360. / (m as f32 + 1.), s, l)
}

const N: usize = 16;
const M: usize = 16;
const W: f32 = 2.;
const TX: f32 = 0.;
const TY: f32 = 10.;

pub fn sort() -> Slide {
    let mut s = Slide::new(80., 60.);
    let ox = 10.;
    let oy = N as f32 + 1.;
    let mut values = [(Referer::Shape(0), Referer::Shape(0)); N];
    for i in 0..N {
        let h = M - i;
        let h = (h as f32) * W;
        values[i].0 =
            shape!(@s, TX, TY-(M as f32 -1.)*W, W, h, z=(N+1), c=rainbow(i, N, 100., 50.));
        values[i].1 = shape!(@s, ox+W*i as f32, oy+W, W, h, z=(1), c=rainbow(i, N, 100., 50.));
        anim!(@s, values[i].0 => Disappear);
    }
    let mut sg = [[(Referer::Shape(0), Referer::Shape(0)); N]; N];
    let mut cc = [Referer::Shape(0); N];
    for i in 0..N {
        let x = ox + W * i as f32;
        let c = shape!(@s, x, oy, W, W, z=(-2));
        cc[i] = c;
        anim!(@s, c => Place, on=c);
        for j in 0..N {
            let set = shape!(@s, x, oy-1.-j as f32, W/2., 1, z=(N-i), c=rainbow(i, N, 100., 70.));
            let get =
                shape!(@s, x+W/2., oy-1.-j as f32, W/2., 1, z=(-1), c=rainbow(i, N, 50., 50.));
            sg[i][j] = (set, get);
            if i != j {
                anim!(@s, get => Disappear);
            }
            // if i == 0 {
            //     anim!(@S, s => Disappear);
            // }
            anim!(@s, set => Appear, on=values[j].0);
            anim!(@s, get => Appear, on=set);
            anim!(@s, values[j].0 => Appear, on=get);
            anim!(@s, get => Disappear, on=get);
            anim!(@s, values[j].0 => Disappear, on=set);
            anim!(@s, values[j].1 => Target(x, oy+W-0.75), on=set);
            anim!(@s, set => Target(TX, TY), on=c);
            anim!(@s, get => Target(TX, TY), on=c);
        }
        for (s0, _) in sg[i] {
            for (s1, g1) in sg[i] {
                anim!(@s, s1 => SlideOut, on=s0);
                anim!(@s, g1 => Place, on=s0);
            }
        }
    }
    for i in 0..N {
        for j in 0..N {
            for k in 0..N {
                anim!(@s, sg[j][i].0 => Disappear, on=sg[k][i].0);
            }
        }
    }

    let stop = shape!(@s, TX, TY, W, W, c=(255, 0, 0));
    let shift = shape!(@s, TX, TY, W, W, z=(N*2+1));
    let run = shape!(@s, TX, TY, W, W, z=(N*2+2), c=(0, 255, 0));
    for i in 0..(N - 1) {
        anim!(@s, shift => Appear, on=cc[i]);
        for j in 0..(N - i - 1) {
            anim!(@s, cc[j] => Target(TX, TY), c=true, on=run);
            anim!(@s, cc[j+1] => Target(TX, TY), on=run);
        }
    }
    anim!(@s, stop => Appear, c=true, on=run);
    anim!(@s, stop => Appear, on=stop);

    for i in 0..(M - 1) {
        for j in 0..N {
            anim!(@s, values[j].0 => Path(0., W*(i+1) as f32), c=j==0, on=shift);
        }
    }
    anim!(@s, shift => Disappear, c=true, on=shift);
    for j in 0..N {
        anim!(@s, values[j].0 => Place, on=shift);
    }
    s
}
