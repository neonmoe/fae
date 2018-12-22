use crate::renderer::Renderer;
use test::Bencher;

#[bench]
fn bench_draw_quad(b: &mut Bencher) {
    let mut renderer = Renderer::create(false);
    let draw_call = renderer.create_dummy_draw_call();
    b.iter(|| {
        renderer.draw_quad(
            (0.0, 0.0, 10.0, 10.0),
            (0.3, 0.3, 0.6, 0.6),
            (1.0, 1.0, 0.5, 1.0),
            (45.0, 5.0, 5.0),
            0.0,
            draw_call,
        );
    });
}

#[bench]
fn bench_draw_quad_legacy(b: &mut Bencher) {
    let mut renderer = Renderer::create(true);
    let draw_call = renderer.create_dummy_draw_call();
    b.iter(|| {
        renderer.draw_quad(
            (0.0, 0.0, 10.0, 10.0),
            (0.3, 0.3, 0.6, 0.6),
            (1.0, 1.0, 0.5, 1.0),
            (45.0, 5.0, 5.0),
            0.0,
            draw_call,
        );
    });
}

#[bench]
fn bench_draw_quad_ninepatch(b: &mut Bencher) {
    let mut renderer = Renderer::create(false);
    let draw_call = renderer.create_dummy_draw_call();
    b.iter(|| {
        renderer.draw_quad_ninepatch(
            ((0.33, 0.33, 0.33), (0.33, 0.33, 0.33)),
            (0.0, 0.0, 10.0, 10.0),
            (0.3, 0.3, 0.6, 0.6),
            (1.0, 1.0, 0.5, 1.0),
            0.0,
            draw_call,
        );
    });
}
