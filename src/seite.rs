// SPDX-FileCopyrightText: 2026 Oliver Weber
// SPDX-License-Identifier: GPL-3.0-or-later

//! Eine PDF-Seite als Qt-Quick-Element, das Rust selbst zeichnet.
//! `paint` läuft im Render-Thread von Qt Quick, während der GUI-Thread wartet;
//! gemeinsame Daten liegen deshalb hinter Mutexen in `engine`/`zustand`.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qrect.h");
        type QRect = cxx_qt_lib::QRect;
        include!("cxx-qt-lib/qrectf.h");
        type QRectF = cxx_qt_lib::QRectF;
        include!("cxx-qt-lib/qsizef.h");
        type QSizeF = cxx_qt_lib::QSizeF;
        include!("cxx-qt-lib/qpainter.h");
        type QPainter = cxx_qt_lib::QPainter;
    }

    unsafe extern "C++" {
        include!(<QtQuick/QQuickPaintedItem>);
        type QQuickPaintedItem;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qml_element]
        #[qobject]
        #[base = QQuickPaintedItem]
        #[qproperty(i32, seite)]
        /// Bildschirm-Pixel je logischem Pixel (Screen.devicePixelRatio)
        #[qproperty(f64, dichte)]
        type Seite = super::SeiteRust;

        #[cxx_name = "paint"]
        #[cxx_override]
        unsafe fn paint(self: Pin<&mut Seite>, painter: *mut QPainter);

        #[inherit]
        fn size(self: &Seite) -> QSizeF;

        #[inherit]
        fn update(self: Pin<&mut Seite>);

        /// Neu zeichnen, z.B. wenn neue Suchtreffer da sind
        #[qinvokable]
        fn neu_zeichnen(self: Pin<&mut Seite>);
    }

    // Parameterlos: QQuickPaintedItem erwartet QQuickItem* statt QObject*, QML setzt das Elternelement selbst
    impl cxx_qt::Constructor<()> for Seite {}
    impl cxx_qt::Threading for Seite {}
}

use core::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use cxx_qt::{CxxQtThread, CxxQtType, Threading};
use cxx_qt_lib::{QColor, QImage, QImageFormat, QPainterRenderHint, QRect, QRectF};

use crate::engine::{self, cache};
use crate::zustand::{self, SUCHE};

#[derive(Default)]
pub struct SeiteRust {
    seite: i32,
    dichte: f64,
    /// zuletzt gewünschte (Seite, Breite); veraltete Render-Aufträge überspringt der PDF-Thread
    wunsch: Arc<AtomicU64>,
    // Box: CxxQtThread ist nicht verschiebbar (!Unpin), so bleibt SeiteRust Unpin
    thread: Option<Box<CxxQtThread<qobject::Seite>>>,
}

fn wunsch_wert(seite: i32, breite_px: i32, drehung: i32) -> u64 {
    ((seite as u32 as u64) << 32) | ((breite_px as u64 & 0xff_ffff) << 8) | (drehung / 90) as u64
}

extern "C" fn arc_freigeben(info: *mut cxx_qt_lib::c_void) {
    // SAFETY: info stammt aus Box::into_raw(Box<Arc<Vec<u8>>>) in `als_qimage`; Qt ruft das genau einmal
    drop(unsafe { Box::from_raw(info.cast::<Arc<Vec<u8>>>()) });
}

/// QImage, das direkt auf die gecachten RGBA-Daten zeigt (keine Kopie); hält dafür einen Arc
fn als_qimage(daten: &Arc<Vec<u8>>, breite: i32, hoehe: i32) -> QImage {
    let halter = Box::new(Arc::clone(daten));
    let zeiger = halter.as_ptr();
    let info = Box::into_raw(halter).cast::<cxx_qt_lib::c_void>();
    // SAFETY: zeiger zeigt auf breite*hoehe*4 Bytes RGBA, die der Arc in `info` am Leben hält
    unsafe { QImage::from_raw_parts(zeiger, breite, hoehe, QImageFormat::Format_RGBA8888, arc_freigeben, info) }
}

impl cxx_qt::Initialize for qobject::Seite {
    fn initialize(mut self: Pin<&mut Self>) {
        let thread = self.qt_thread();
        self.as_mut().rust_mut().get_mut().thread = Some(Box::new(thread));
    }
}

impl qobject::Seite {
    fn neu_zeichnen(self: Pin<&mut Self>) {
        self.update();
    }

    unsafe fn paint(self: Pin<&mut Self>, painter: *mut qobject::QPainter) {
        // SAFETY: Qt übergibt einen gültigen QPainter für die Dauer von paint()
        let Some(painter) = (unsafe { painter.as_mut() }) else { return };
        // SAFETY: der QPainter gehört Qt und wird während paint() weder verschoben noch freigegeben
        let mut p = unsafe { Pin::new_unchecked(painter) };
        let groesse = self.size();
        let (w, h) = (groesse.width(), groesse.height());
        if w < 1.0 || h < 1.0 {
            return;
        }
        let ziel = QRect::new(0, 0, w.round() as i32, h.round() as i32);
        let seite = *self.seite();
        let dok = zustand::dok_nr();
        let drehung = zustand::drehung_von(seite);
        let dichte = self.dichte().max(1.0);
        let breite_px = (w * dichte).round() as i32;
        // PDFium rendert ungedreht: bei 90/270 Grad ist die ungedrehte Breite die angezeigte Höhe
        let breite_ungedreht = if drehung % 180 == 0 { breite_px } else { (h * dichte).round() as i32 };
        let key = (dok, seite, breite_px, drehung);
        self.rust().wunsch.store(wunsch_wert(seite, breite_px, drehung), Ordering::Release);

        let (bild, ersatz, neu_anfordern) = {
            let mut c = cache().lock().unwrap();
            match c.holen(key) {
                Some(b) => (Some(b), None, false),
                None => {
                    let ersatz = c.naechstbeste(dok, seite, breite_px, drehung);
                    (None, ersatz, c.vormerken(key))
                }
            }
        };

        match bild.or(ersatz) {
            Some((daten, bw, bh)) => {
                p.as_mut().set_render_hint(QPainterRenderHint::SmoothPixmapTransform, true);
                p.as_mut().draw_image(&ziel, &als_qimage(&daten, bw, bh));
            }
            None => p.as_mut().fill_rect(&QRectF::new(0.0, 0.0, w, h), &QColor::from_rgb(255, 255, 255)),
        }

        if neu_anfordern {
            self.anfordern(key, breite_ungedreht);
        }

        // Suchtreffer: gelb, der ausgewählte orange
        let s = SUCHE.lock().unwrap();
        let gelb = QColor::from_rgba_f(1.0, 0.85, 0.0, 0.38);
        let orange = QColor::from_rgba_f(1.0, 0.45, 0.0, 0.55);
        for i in s.bereich(seite) {
            let farbe = if i as i64 == s.aktuell { &orange } else { &gelb };
            for r in &s.treffer[i].rects {
                let r = zustand::gedreht(*r, drehung);
                let rect = QRectF::new(r.x as f64 * w, r.y as f64 * h, r.b as f64 * w, r.h as f64 * h);
                p.as_mut().fill_rect(&rect, farbe);
            }
        }
    }

    fn anfordern(&self, key: engine::Schluessel, breite_ungedreht: i32) {
        let (dok, seite, breite_px, drehung) = key;
        let (Ok(engine), Some(thread)) = (engine::engine(), self.rust().thread.as_deref().cloned()) else {
            cache().lock().unwrap().fehlgeschlagen(key);
            return;
        };
        let wunsch = Arc::clone(&self.rust().wunsch);
        let pruef_thread = thread.clone();
        engine.rendern(
            dok,
            seite,
            breite_ungedreht,
            drehung,
            Box::new(move || {
                !pruef_thread.is_destroyed() && wunsch.load(Ordering::Acquire) == wunsch_wert(seite, breite_px, drehung)
            }),
            Box::new(move |bild| match bild {
                Some(bild) => {
                    cache().lock().unwrap().ablegen(key, bild);
                    let _ = thread.queue(|s| s.update());
                }
                None => cache().lock().unwrap().fehlgeschlagen(key),
            }),
        );
    }
}
