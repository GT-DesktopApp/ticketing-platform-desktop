// Placeholder shell — intentionally minimal.
//
// The frontend UI is NOT built yet. Per the current phase we focus on the
// src-tauri backend (CRUD + core). Real screens/components will be implemented
// here once the Figma designs are available.
//
// The Ticket API client and types already exist (src/api, src/lib) so the UI
// can be wired up quickly later, but nothing is rendered yet.

export default function App() {
  return (
    <main className="placeholder">
      <p className="placeholder__eyebrow">Ticketing Platform</p>
      <h1>Ticketing Platform — Desktop</h1>
      <p className="placeholder__note">
        Foundation scaffold. Backend (Tauri + SQLite) under development; UI
        pending Figma designs.
      </p>
    </main>
  );
}
