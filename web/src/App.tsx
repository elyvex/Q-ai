// Q-ai reading UI shell (Phase 5 scaffold). Offline by construction: no
// CDN, no webfonts, no runtime URL outside `/` and `/api/`. The reading and
// research views land in plans 05-06 and 05-09; the default font is the
// system Arabic stack (OD-04 stays open — no bundled webfont).
export default function App() {
  return (
    <main
      style={{
        fontFamily:
          "'Amiri', 'Scheherazade New', 'Noto Naskh Arabic', 'Geeza Pro', serif",
        direction: 'rtl',
        padding: '2rem',
      }}
    >
      <h1>Q-ai</h1>
      <p>Quran research cockpit — reading view ships in plan 05-06.</p>
    </main>
  )
}
