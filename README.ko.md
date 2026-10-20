<p align="center">
  <img src="https://raw.githubusercontent.com/heonny/egui-glass/main/assets/branding/readme-logo.png" alt="egui_glass" width="640">
</p>

[English](https://github.com/heonny/egui-glass/blob/main/README.md) | **한국어**

[![CI](https://github.com/heonny/egui-glass/actions/workflows/ci.yml/badge.svg)](https://github.com/heonny/egui-glass/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/egui_glass.svg)](https://crates.io/crates/egui_glass)
[![API docs](https://docs.rs/egui_glass/badge.svg)](https://docs.rs/egui_glass)

Apple의 *Liquid Glass*에서 영감을 받은 [egui](https://github.com/emilk/egui)용 글라스
표면입니다. 가장자리 굴절, 블러, 틴트, 얇은 베벨 하이라이트를 `egui-wgpu`를 통해 GPU에서
그립니다. 컨테이너, 캡슐 버튼, 글라스 슬라이더, 툴바, 라이트/다크 프리셋을 제공하고,
egui 콘텐츠 자체를 굴절시키는 라이브 백드롭도 선택적으로 쓸 수 있습니다. 개발 버전에서는
버튼과 슬라이더의 호버/프레스 상태가 짧은 재질 전환 애니메이션으로 바뀌고, 릴리스된
0.1.5에서는 즉시 바뀝니다.

![demo](https://raw.githubusercontent.com/heonny/egui-glass/main/docs/demo.png)

## 빠른 시작

Rust 1.92 이상, egui/eframe 0.35, wgpu 29가 필요합니다. 렌더러는 **wgpu**를 써야 하며
glow는 지원하지 않습니다. 아래 `wayland`, `x11` 피처는 Linux 창 생성을 위한 것입니다.
wgpu를 지원하는 그래픽 어댑터가 있어야 합니다. `cargo new glass-app`으로 프로젝트를 만들고
의존성을 아래 내용으로 바꾼 뒤, Rust 예제를 `src/main.rs`에 넣고 `cargo run`을 실행하세요.
별도 에셋이나 이 저장소의 체크아웃은 필요 없습니다.

```toml
[dependencies]
eframe = { version = "0.35", default-features = false, features = ["accesskit", "default_fonts", "wgpu", "wayland", "x11"] }
egui = "0.35"
egui_glass = "0.1.5"
```

```rust
use eframe::egui;
use egui_glass::{Glass, GlassToolbar};

struct App;

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("eframe must use the wgpu renderer");
        egui_glass::init(rs, 1);
        // A gradient makes refraction visible; a uniform colour looks almost flat.
        let mut wallpaper = egui::ColorImage::filled([256, 256], egui::Color32::WHITE);
        for y in 0..256 {
            for x in 0..256 {
                wallpaper[(x, y)] = egui::Color32::from_rgb(70 + x as u8 / 2, 130, 180 + y as u8 / 4);
            }
        }
        egui_glass::set_backdrop(&cc.egui_ctx, rs, &wallpaper);   // what the glass refracts
        Self
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            egui_glass::show_backdrop(ui, ui.max_rect());
            egui::Area::new(egui::Id::new("card"))
                .default_pos(egui::pos2(40.0, 40.0)).movable(true).constrain(false)
                .show(ui.ctx(), |ui| {
                Glass::default().show(ui, |ui| {
                    ui.heading("Hello glass");
                    ui.label("Drag this card to see the background change.");
                    let _ = ui.button("Continue"); // ordinary controls inside glass
                });
            });
            egui::Area::new(egui::Id::new("bar"))
                .default_pos(egui::pos2(40.0, 200.0)).movable(true).constrain(false)
                .show(ui.ctx(), |ui| {
                GlassToolbar::default().show(ui, |ui| {
                    let _ = ui.button("Undo");
                    let _ = ui.button("Share");
                });
            });
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions { renderer: eframe::Renderer::Wgpu, ..Default::default() };
    eframe::run_native("glass", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
```

`init`은 렌더러마다 한 번만 호출합니다. `set_backdrop`은 처음 한 번, 그리고 이미지가
바뀔 때마다 호출해 업로드합니다. `show_backdrop`은 매 프레임 글라스를 그리기 전에
호출합니다. `init`에 넘긴 `1`은 기본 MSAA 설정에 맞춘 값입니다
(`NativeOptions::multisampling = 0`이 샘플 1개를 뜻합니다). MSAA를 켰다면 양쪽에 같은 샘플
수를 넘기세요. 렌더러는 UI가 살아 있는 동안 계속 유지해야 합니다.

## 글라스에 비칠 대상 고르기

| 백드롭 | 글라스 너머로 보이는 것 | 비용과 용도 |
|---|---|---|
| 정적 이미지 (위 예제) | 해당 화면 위치에 등록된 이미지만 | 업로드 한 번. 그라디언트나 배경화면 위의 사이드바, 카드, 다이얼로그에 적합 |
| `LiveBackdrop` | 클로저가 그린 페이지 콘텐츠 (텍스트 포함) | 매 프레임 레이아웃과 GPU 렌더링이 추가됨. 변하는 콘텐츠 위의 플로팅 컨트롤에 적합 |

예를 들어 정적 배경화면 위에 로그 한 줄을 그렸다면, 그 위의 글라스에는 흐려진 로그가 아니라
여전히 **배경화면**이 비칩니다. `blur` 값을 올려도 마찬가지입니다. 정적 글라스는 OS 창
투명도가 아니므로 바탕화면이나 다른 앱을 비출 수 없습니다.

라이브 백드롭을 쓰려면 앱 상태에 `LiveBackdrop`을 보관하세요. `init` 다음, `set_backdrop`이나
네이티브 텍스처 등록보다는 먼저 만듭니다. 이것이 정적 백드롭 그리기 단계를 대신하며, 글라스는
페이지 클로저 바깥에 둡니다:

```rust
// In the creation closure, after init (rs is the wgpu render state):
let live = egui_glass::LiveBackdrop::new(rs, None); // default fonts

// In App::ui, using the stored live value:
live.run(ui, egui::Color32::WHITE, |page| {
    page.label("This text is part of the refracted page.");
});
egui::Area::new(egui::Id::new("overlay"))
    .fixed_pos(egui::pos2(20.0, 20.0)).constrain(false)
    .show(ui.ctx(), |ui| {
        egui_glass::Glass::panel().show(ui, |ui| { ui.label("Floating glass"); });
    });
```

페이지 클로저는 화면용 레이아웃과 오프스크린 레이아웃에서 각각 실행되고, egui가 패스를
추가로 요청하면 또 실행될 수 있습니다. 파일 쓰기, 네트워크 호출, 상태 전이 같은 부수 효과는
클로저 바깥에 두세요. 라이브 백드롭은 렌더러마다 하나만 만듭니다. 커스텀 폰트를 쓴다면
`Some(font_definitions)`를 넘기고 메인 컨텍스트와 같은 폰트를 유지하세요. 페이지 안의
이미지는 `egui_glass::register_native_texture`로 등록한 텍스처여야 합니다 (`set_backdrop`은
자기 이미지를 알아서 등록합니다). 일반적인 egui 텍스처 업로드는 자동으로 복제되지 않습니다.

개발 버전에는 프레임마다 라이브 백드롭 품질을 고르는 기능이 추가되었습니다 (미릴리스):

```rust
live.run_with_quality(ui, egui::Color32::WHITE, egui_glass::LiveBackdropQuality::Balanced, |page| {
    page.label("This text is part of the refracted page.");
});
```

`Full`은 원래 해상도, `Balanced`는 가로세로 0.75배, `Performance`는 0.5배로 렌더링합니다.
기존 `run`은 `Full`로 동작합니다. 낮은 품질을 고르면 화면에 보이는 페이지와 글라스 형태는
원래 해상도를 유지한 채 오프스크린 픽셀과 밉맵 작업만 줄어듭니다. 굴절되는 작은 글자는
조금 더 흐려지고, UI 레이아웃이 한 번 더 도는 비용은 그대로입니다. 측정 범위와 로컬 결과는
[벤치마크 안내](https://github.com/heonny/egui-glass/blob/main/docs/performance.md)를 참고하세요.

## 기존 앱에 글라스 넣기

초기화와 의존성은 위와 같습니다. 내비게이션 영역이나 잠깐 떠 있는 오버레이부터 적용하고,
대비가 중요한 빽빽한 텍스트와 코드는 불투명한 표면에 두세요. 글라스는 앱의 이벤트 처리나
저장 로직을 건드리지 않습니다.

### 사이드바와 불투명 콘텐츠

이 코드는 `App::ui`에 들어갑니다. 패널을 배치하기 **전에** 루트 UI 전체에 백드롭을 한 번
그려야 모든 표면이 같은 화면 좌표계를 공유합니다. 바깥 패널 프레임을 투명하게 두면 그
백드롭이 비칩니다. `Glass`에는 자체 패딩이 있으므로 (기본 16 포인트) 여백을 두 겹으로
원하는 게 아니라면 패딩이 있는 프레임을 하나 더 두지 마세요.

```rust
egui_glass::show_backdrop(ui, ui.max_rect());
egui::Panel::left("navigation")
    .default_size(220.0)
    .frame(egui::Frame::NONE)
    .show(ui, |ui| {
        egui_glass::Glass::panel().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            ui.set_min_size(ui.available_size()); // fill the panel, even with little content
            ui.heading("Projects");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for name in ["Frontend", "API", "Worker"] {
                    let _ = ui.button(name);
                }
            });
        });
    });
egui::CentralPanel::default()
    .frame(egui::Frame::new().fill(egui::Color32::WHITE))
    .show(ui, |ui| { ui.label("Opaque log or editor content"); });
```

### 스크롤되고 닫히는 모달

모달의 열림 플래그와 액션 처리는 기존 코드를 그대로 씁니다. 모달에 `Frame::NONE`을 주고
그 안에 글라스를 넣으면, 입력 차단과 바깥 클릭/Escape로 닫기는 모달이 계속 담당합니다.
안쪽 `ScrollArea`가 폼을 스크롤하고 푸터는 항상 보입니다.

```rust
// In App::ui, after drawing the page. `open` is your &mut bool modal state.
if *open {
    let response = egui::Modal::new(egui::Id::new("settings"))
        .frame(egui::Frame::NONE)
        .show(ui.ctx(), |ui| {
            egui_glass::Glass::panel().inner_margin(egui::Margin::same(20)).show(ui, |ui| {
                ui.set_width(320.0);
                ui.heading("Settings");
                let body_height = (ui.ctx().content_rect().height() - 160.0).max(80.0);
                egui::ScrollArea::vertical().max_height(body_height).show(ui, |ui| {
                    for n in 1..=20 { ui.label(format!("Setting {n}")); }
                });
                ui.button("Close").clicked()
            }).inner
        });
    if response.should_close() || response.inner { *open = false; }
}
```

### 기존 컨트롤 테마 유지하기

`Glass::show`와 `GlassToolbar::show`는 자식 컨트롤에 평평한 비주얼을 입힙니다. 유휴 상태는
투명하고, 모서리는 캡슐형이며, 전경색과 선택색이 바뀌고, 위젯 테두리가 사라집니다. 이 변경은
자식 UI 안에서만 유효합니다. 개발 버전에서는 (미릴리스) `.preserve_theme(true)`로 앱 고유의
버튼, 검증 색상, 포커스 스타일을 그대로 둘 수 있습니다:

```rust
egui_glass::Glass::panel().preserve_theme(true).show(ui, |ui| {
    let _ = ui.button("Uses the app's theme");
});
```

`GlassToolbar`에도 같은 옵션이 있습니다. 기본값은 `false`이며 평평한 자식 비주얼을 씁니다.
이 옵션이 보존하는 것은 컨트롤 비주얼뿐이고, 글라스 재질, 패딩, 툴바 간격은 여전히 글라스
컨테이너가 정합니다. 릴리스된 **0.1.5**에서는 대신 컨테이너에 들어가기 **전에** 스타일을
복사해 두었다가 안에서 되돌리세요:

```rust
let app_style = ui.style().clone();
egui_glass::Glass::panel().show(ui, |ui| {
    ui.set_style(app_style);
    let _ = ui.button("Uses the app's theme");
});
```

되돌린 텍스트 색에 어울리는 글라스 틴트를 고르세요. 다크 글라스 위에 라이트 테마를 되돌리면
그 테마의 어두운 글자색까지 함께 돌아옵니다. 글라스 컨테이너 안에서는 일반 egui 컨트롤을
쓰세요. `GlassButton`은 그 자체로 또 하나의 글라스 표면이므로, `Glass`나 `GlassToolbar` 안에
넣지 말고 백드롭 위에 직접 두세요.

## 재질과 컨트롤

| API / 프리셋 | 용도 |
|---|---|
| `Glass::default()` / `GlassStyle::regular()` | 작은 반투명 카드. 베벨과 후광이 절제됨 |
| `Glass::panel()` / `GlassStyle::panel()` | 큰 시트와 사이드바. 가장자리가 차분함 |
| `GlassStyle::clear()` | 화려한 이미지 위에서 렌즈 효과가 강하고 흐림은 약함 |
| `GlassStyle::dark()` / `panel_dark()` | 다크 버전. 다크 백드롭과 함께 사용 |
| `GlassButton::new("Continue").show(ui)` | 단독으로 쓰는 캡슐 버튼 |
| `GlassToolbar::default().show(ui, closure)` | 일반 egui 컨트롤을 담는 툴바 |
| `GlassSlider::new(&mut value, 0.0..=100.0).show(ui)` | 글라스 썸과 값 편집 칸이 있는 단독 슬라이더 |
| `paint_glass(ui, rect, &style)` | 직접 그리는 표면. 전경 콘텐츠보다 먼저 그릴 것 |

위젯 배치는 그대로 두고 재질만 바꿀 수 있습니다:

```rust
let style = egui_glass::GlassStyle {
    blur: 20.0,
    refraction: 4.0,
    chromatic: 0.0,
    corner_radius: 12.0,
    tint: egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180),
    ..egui_glass::GlassStyle::panel()
};
egui_glass::Glass::new(style).show(ui, |ui| { ui.label("A quiet sheet"); });
```

틴트 알파가 클수록 틴트 색이 진하게 섞입니다. 흰색 틴트를 진하게 하면 글자 대비는 좋아지지만
백드롭은 잘 안 보입니다. 블러와 굴절의 단위는 논리 포인트입니다. 다크 UI라면 다크 프리셋에
어울리는 호스트 테마와 백드롭을 함께 쓰세요. 투명도 줄이기 옵션이 필요하다면 글라스 대신
앱에서 일반 불투명 `egui::Frame`을 그리면 됩니다.

## 문제 해결

개발 버전의 `GlassButton`과 `GlassSlider`는 호버/프레스 재질 변화를 로컬 egui
`Style::animation_time`에 따라 애니메이션하며, 길어도 120 ms입니다. 즉시 바꾸고 싶다면 두
위젯에 `.animate(false)`를 주거나 호스트 애니메이션 시간을 `0.0`으로 두세요. 클릭, 포커스,
슬라이더 값, 썸 위치는 어느 경우든 즉시 반영됩니다.

| 증상 | 확인할 것 |
|---|---|
| `wgpu_render_state`가 없거나 글라스가 안 그려짐 | eframe의 `wgpu` 피처를 켜고 `Renderer::Wgpu`를 선택. glow는 미지원 |
| `set_backdrop`에서 패닉, 또는 GPU 샘플 수 오류 | `init`을 먼저 호출하고 렌더러의 MSAA 샘플 수와 맞출 것 |
| 글라스가 평평하거나 하얗게만 보임 | 단색이 아닌 백드롭을 등록하고 글라스보다 먼저 그릴 것. 필요하면 틴트 알파를 낮춤 |
| 글라스 뒤의 텍스트가 안 비침 | 정적 글라스에는 이미지만 비침. 페이지 콘텐츠는 `LiveBackdrop`으로 |
| 패널 주변 배경이 사라짐 | 불투명한 패널 프레임이 덮는 중. 바깥 프레임에 `Frame::NONE` |
| 버튼이나 선택 색상이 달라짐 | 글라스가 평평한 자식 비주얼을 입힘. 위의 스타일 유지 방법 참고 |
| 다이얼로그 여백이 두 겹 | 모달 바깥 프레임의 패딩을 없애고 `Glass`에만 패딩을 줄 것 |
| 플로팅 영역이 첫 프레임에 튐 | `Area::constrain(false)`를 쓰고, 뷰포트 안에 가두는 일은 앱이 담당 |
| 플로팅 오버레이 아래 페이지가 스크롤 안 됨 | egui는 휠 입력을 맨 위 레이어에만 보냄. 스크롤 콘텐츠를 그 레이어에 두거나 앱에서 휠 입력을 직접 넘길 것 |
| 라이브 글라스 너머로 이미지가 안 보임 | 라이브 페이지를 그리기 전에 네이티브 텍스처를 라이브러리에 등록 |
| 헤드리스 테스트에 GPU 렌더러가 없음 | 콘텐츠는 불투명 프레임으로 테스트하고, 글라스 렌더링은 wgpu로 따로 검증 |

## 도입 전에 알아둘 것

이 크레이트는 **0.1 버전**입니다. 네이티브 egui 앱에서 글라스를 실험하기에 알맞지만, 작은
API가 1.0 전에 바뀔 수 있습니다. 대상 기기에서 렌더링과 조작감을 직접 확인하세요.

- 정적 글라스는 등록된 이미지만 샘플링하며, 뒤에 놓인 위젯을 자동으로 담지 않습니다.
  `LiveBackdrop`은 페이지 콘텐츠를 오프스크린에 한 번 더 그리므로 매 프레임 레이아웃과
  GPU 작업이 늘어납니다.
- 라이브러리 테스트는 CI에서 Linux, macOS, Windows로 돌고, 데모 빌드는 macOS와 Windows에서
  확인합니다. 빌드와 테스트를 확인하는 것이지 GPU 렌더링을 보증하는 것은 아니며, 브라우저와
  모바일은 CI 범위 밖입니다.
- `GlassButton`은 Tab 이동, Enter/Space 활성화, 눈에 보이는 포커스 외곽선, 스크린 리더용
  버튼 이름을 지원합니다. 호스트의 AccessKit 연동을 켜고 (위 의존성 참고), 아이콘만 있는
  버튼에는 `.accessible_name("Back")`을 붙이세요. 불투명 모드나 투명도 줄이기 대체 표현은
  앱이 책임지며, 실제 백드롭 위에서 대비를 확인해야 합니다.
- 기본 크레이트 피처는 없습니다. `GlassStyle`을 직렬화하려면 `features = ["serde"]`를
  켜세요.

## 문서

- [가이드](https://github.com/heonny/egui-glass/blob/main/docs/guide.md) — 기존 앱에 글라스 넣기: 백드롭, 컴포넌트, 프리셋, 다크
  테마, 라이브 콘텐츠 위의 글라스.
- [레퍼런스](https://github.com/heonny/egui-glass/blob/main/docs/reference.md) — 모든 함수와 `GlassStyle` 필드, 셰이더 동작 원리,
  한계, 데모 앱, macOS 앱 빌드, 배포.
- [docs.rs](https://docs.rs/egui_glass) — API 문서.
- [기여](https://github.com/heonny/egui-glass/blob/main/CONTRIBUTING.md) — 개발, 버그 보고, 릴리스 점검.
- [변경 이력](https://github.com/heonny/egui-glass/blob/main/CHANGELOG.md) — 다음 릴리스에 들어갈 변경 사항.
- [검증](https://github.com/heonny/egui-glass/blob/main/docs/verification.md) — 자동 GPU 검사와 남은 네이티브 테스트.
- [성능](https://github.com/heonny/egui-glass/blob/main/docs/performance.md) — 품질 옵션, 벤치마크 명령, 측정 한계.

가이드와 레퍼런스를 비롯한 나머지 문서는 영문이 원본이며, 한국어판은 이 README만 제공합니다.

## 데모

```bash
cargo run -p egui_glass_demo          # sliders for every parameter, presets, export/import
cargo run -p egui_glass_demo --example minimal
make install                           # macOS: Egui Glass.app into /Applications
```

개발 버전 데모에는 현재 재질을 완전한 `egui_glass::GlassStyle` 표현식으로 복사하는
**Copy Rust** 버튼이 있습니다. **Modified**는 선택한 프리셋과 달라진 상태를 표시하고,
**Reset**은 그 프리셋으로 되돌립니다. Import 뒤에는 가져온 재질이 Reset의 기준이 됩니다.
Reset은 재질만 되돌리고 Live 백드롭 토글은 건드리지 않습니다.

## 라이선스

MIT. "Liquid Glass"는 Apple 디자인 언어의 이름이며, 이 프로젝트는 그 외형을 독자적으로
구현한 것으로 Apple과 관계가 없습니다. 데모 이미지와 브랜딩의 출처는
[에셋 출처](https://github.com/heonny/egui-glass/blob/main/docs/assets.md)에 있으며, 이
에셋은 라이브러리 크레이트에 포함되지 않습니다.

### 통합 관리 API (개발 버전)

미릴리스 `GlassContext` API는 초기화와 리소스 관리를 한곳에 모읍니다:

```rust
let glass = egui_glass::GlassContext::new(ctx, rs, 1)?;
glass.set_fonts(font_definitions);
glass.set_backdrop(&wallpaper)?;
// Store `glass` in application state.
```

`init`과 `LiveBackdrop::new`를 대신합니다. 라이브 렌더링은 `glass.live_backdrop()`으로
접근합니다. 두 번 초기화하거나 잘못된 이미지를 넘기면 `GlassError`가 반환되고, 잘못된
업로드는 이전 백드롭을 그대로 둡니다. `glass.register_native_texture(&view)`로 등록한
네이티브 텍스처는 복제할 수 있는 소유 핸들로 돌아옵니다. 렌더링하는 동안 핸들을 붙들고
있다가, 마지막 핸들이 사라질 때 양쪽 렌더러에서 등록이 해제되게 하세요. 수명과 MSAA 요구
사항은
[통합 관리 API 레퍼런스](https://github.com/heonny/egui-glass/blob/main/docs/reference.md#managed-setup-development-version)를
보세요. 위의 0.1.5 통합 예제도 그대로 유효합니다.
