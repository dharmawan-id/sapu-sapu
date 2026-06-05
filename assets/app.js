// Sapu Sapu landing: English/Indonesian toggle. Default English is inline in
// the HTML so the page reads fine without JavaScript; this swaps to Indonesian.
(function () {
  const EN = {
    brandSub: "Disk Cleaner",
    navRepo: "Source",
    navDownload: "Download",
    heroEyebrow: "Windows disk cleaner",
    heroTitleA: "Clean the",
    heroTitleB: "tank.",
    heroLede:
      "Named after the sapu-sapu, the janitor fish that keeps the tank glass clear. It scans C: and D:, shows where the space went, and cleans the safe caches. Preview first, real reclaim, nothing important touched.",
    ctaDownload: "Download for Windows",
    ctaSource: "View source",
    shotCap: "The overview: drive gauges, biggest folders and files, breakdown by type.",
    featK: "What it does",
    featTitle: "Two jobs, done plainly",
    f1t: "Where the space went",
    f1b: "A parallel scan of a whole drive: the biggest folders, the biggest files, and a breakdown by file type, for C: and D:.",
    f2t: "Honest numbers",
    f2b: "Freed space is measured from the real free-space change, so a hardlinked cache reports the few gigabytes it actually frees, not an inflated total.",
    f3t: "Preview, then clean",
    f3b: "Nothing is deleted until you select it. Files held by a running program are skipped. Recent or uncommitted project folders are flagged unsafe.",
    f4t: "Hands off what matters",
    f4b: "The Windows Installer store, browser profiles, the global npm prefix, SSH and cloud keys. The guard lives in Rust, so the interface cannot override it.",
    safeK: "Safety",
    safeTitle: "Three tiers",
    tg: "Green",
    tgt: "Regenerating caches",
    tgb: "uv, npm, pip, cargo, browser, HuggingFace, VS Code, Temp. Previewed, then cleared on confirm. They rebuild themselves.",
    ty: "Yellow",
    tyt: "Project artifacts",
    tyb: "node_modules, target, dist, build, .next, __pycache__. Per item, and flagged unsafe if touched in the last 30 days or sitting in a repo with uncommitted changes.",
    tr: "Protected",
    trt: "Never touched",
    trb: "Installer store, active sandbox, npm global prefix, browser profiles, SSH and cloud keys. Refused even if asked.",
    howK: "How it works",
    howTitle: "Scan, preview, clean",
    s1t: "Scan",
    s1b: "Read a drive or list the caches. Sizes come back with file counts.",
    s2t: "Preview",
    s2b: "See exactly what will be removed and how much it holds. Pick per item.",
    s3t: "Clean",
    s3b: "Selected items are removed. The freed total is the real free-space change.",
    closerNote:
      "Sapu Sapu runs without admin. It needs the WebView2 runtime, which ships with current Windows 10 and 11. It is open source under the MIT License.",
    clDownload: "Download",
    clSource: "Source",
    clReadme: "Readme",
    footAuthor: "Built by Dharmawan.",
    footLicense: "MIT License. Neo-brutalist, timber accent.",
  };

  const ID = {
    brandSub: "Pembersih Disk",
    navRepo: "Sumber",
    navDownload: "Unduh",
    heroEyebrow: "Pembersih disk Windows",
    heroTitleA: "Bersihkan",
    heroTitleB: "kaca.",
    heroLede:
      "Dinamai dari ikan sapu-sapu yang menjaga kaca akuarium tetap bening. Ia memindai C: dan D:, menunjukkan ke mana ruang habis, dan membersihkan cache yang aman. Pratinjau dulu, hasil nyata, tidak menyentuh yang penting.",
    ctaDownload: "Unduh untuk Windows",
    ctaSource: "Lihat sumber",
    shotCap: "Tampilan ringkas: gauge drive, folder dan file terbesar, rincian per tipe.",
    featK: "Yang ia kerjakan",
    featTitle: "Dua tugas, dikerjakan apa adanya",
    f1t: "Ke mana ruang habis",
    f1b: "Pemindaian paralel satu drive penuh: folder terbesar, file terbesar, dan rincian per tipe berkas, untuk C: dan D:.",
    f2t: "Angka yang jujur",
    f2b: "Ruang yang dibebaskan diukur dari perubahan ruang kosong yang sebenarnya, jadi cache ber-hardlink melaporkan beberapa gigabyte yang benar-benar ia bebaskan, bukan total yang menggelembung.",
    f3t: "Pratinjau, baru bersihkan",
    f3b: "Tidak ada yang dihapus sebelum Anda memilihnya. Berkas yang sedang dipakai program dilewati. Folder proyek yang baru atau belum di-commit ditandai tidak aman.",
    f4t: "Tidak menyentuh yang penting",
    f4b: "Penyimpanan Windows Installer, profil browser, prefix npm global, kunci SSH dan cloud. Penjaganya ada di Rust, jadi antarmuka tidak bisa menimpanya.",
    safeK: "Keamanan",
    safeTitle: "Tiga tingkat",
    tg: "Hijau",
    tgt: "Cache yang dibuat ulang",
    tgb: "uv, npm, pip, cargo, browser, HuggingFace, VS Code, Temp. Dipratinjau, lalu dibersihkan atas konfirmasi. Semuanya terbentuk lagi sendiri.",
    ty: "Kuning",
    tyt: "Artefak proyek",
    tyb: "node_modules, target, dist, build, .next, __pycache__. Per item, ditandai tidak aman jika disentuh dalam 30 hari terakhir atau berada di repo dengan perubahan belum di-commit.",
    tr: "Terlindungi",
    trt: "Tidak pernah disentuh",
    trb: "Penyimpanan Installer, sandbox aktif, prefix npm global, profil browser, kunci SSH dan cloud. Ditolak meski diminta.",
    howK: "Cara kerjanya",
    howTitle: "Pindai, pratinjau, bersihkan",
    s1t: "Pindai",
    s1b: "Baca satu drive atau daftarkan cache-nya. Ukuran kembali lengkap dengan jumlah berkas.",
    s2t: "Pratinjau",
    s2b: "Lihat persis apa yang akan dihapus dan seberapa besar. Pilih per item.",
    s3t: "Bersihkan",
    s3b: "Item terpilih dihapus. Total yang dibebaskan adalah perubahan ruang kosong yang sebenarnya.",
    closerNote:
      "Sapu Sapu berjalan tanpa admin. Ia butuh runtime WebView2 yang sudah ada di Windows 10 dan 11 terkini. Sumber terbuka di bawah Lisensi MIT.",
    clDownload: "Unduh",
    clSource: "Sumber",
    clReadme: "Readme",
    footAuthor: "Dibuat oleh Dharmawan.",
    footLicense: "Lisensi MIT. Neo-brutalis, aksen timber.",
  };

  function apply(dict) {
    document.querySelectorAll("[data-i18n]").forEach((el) => {
      const key = el.getAttribute("data-i18n");
      if (dict[key] != null) el.textContent = dict[key];
    });
  }

  document.querySelectorAll(".lang__btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const lang = btn.getAttribute("data-lang");
      apply(lang === "id" ? ID : EN);
      document.documentElement.lang = lang;
      document.querySelectorAll(".lang__btn").forEach((b) => {
        const on = b === btn;
        b.classList.toggle("is-active", on);
        b.setAttribute("aria-pressed", String(on));
      });
    });
  });
})();
