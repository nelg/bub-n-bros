// Download helper: picks the right release file from three or four answers.
// The file list is in releases.js.

(function () {
  "use strict";

  const $ = (id) => document.getElementById(id);
  const fileUrl = (name) => DOWNLOAD_BASE + name;

  // ---------------------------------------------------------------- detection

  function detectSystem() {
    const ua = navigator.userAgent || "";
    const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    const p = (platform + " " + ua).toLowerCase();
    if (/android|iphone|ipad|ipod/.test(p)) return { os: "", note: "a phone or tablet (the game needs a computer)" };
    if (p.includes("win")) return { os: "windows", note: "Windows" };
    if (p.includes("mac")) return { os: "macos-arm", note: "macOS", mac: true };
    if (/linux|x11|cros/.test(p)) {
      let distro = "";
      if (/ubuntu/.test(p)) distro = "ubuntu";
      else if (/fedora/.test(p)) distro = "fedora";
      else if (/debian/.test(p)) distro = "debian";
      return { os: "linux", distro: distro, note: "Linux" + (distro ? " (" + $("distro").querySelector('option[value="' + distro + '"]').textContent + ")" : "") };
    }
    return { os: "", note: "" };
  }

  // Chromium browsers can tell Apple Silicon from Intel Macs.
  function refineMac(apply) {
    if (!navigator.userAgentData || !navigator.userAgentData.getHighEntropyValues) return;
    navigator.userAgentData.getHighEntropyValues(["architecture"]).then((v) => {
      if (v.architecture === "x86") apply("macos-intel", "macOS on an Intel processor");
      else if (v.architecture === "arm") apply("macos-arm", "macOS on Apple Silicon");
    }).catch(() => {});
  }

  // ---------------------------------------------------------------- answers

  function answers() {
    const radio = (name) => {
      const el = document.querySelector('input[name="' + name + '"]:checked');
      return el ? el.value : "";
    };
    return { os: $("os").value, distro: $("distro").value, python: radio("python"), use: radio("use") };
  }

  // ---------------------------------------------------------------- recommendations

  const PKG = {
    ubuntu: { suffix: "_2.4.0-1_amd64-ubuntu-24.04.deb", sep: "_", install: (f) => "sudo apt install ./" + f, name: "Ubuntu" },
    debian: { suffix: "_2.4.0-1_amd64-debian-trixie.deb", sep: "_", install: (f) => "sudo apt install ./" + f, name: "Debian" },
    fedora: { suffix: "-2.4.0-1.fc45.x86_64.fedora.rpm", sep: "-", install: (f) => "sudo dnf install ./" + f, name: "Fedora" },
    openmandriva: { suffix: "-2.4.0-1-omv2590.znver1.openmandriva.rpm", sep: "-", install: (f) => "sudo dnf install ./" + f, name: "OpenMandriva" },
  };

  function linuxPackage(pkg, distro) {
    const d = PKG[distro];
    return pkg + d.suffix;
  }

  const code = (s) => "<pre><code>" + s.replace(/&/g, "&amp;").replace(/</g, "&lt;") + "</code></pre>";

  function archSteps(pkg) {
    return [
      "Arch Linux has no ready-made package in the release; build it with the recipe in the source code (this builds all three packages; install the one you want):" +
      code("sudo pacman -S --needed git base-devel\n" +
           "git clone --branch v" + VERSION + " https://github.com/nelg/bub-n-bros.git\n" +
           "cd bub-n-bros/bub-n-bros/packing/arch\n" +
           "cp PKGBUILD.txt PKGBUILD\n" +
           "makepkg -s\n" +
           "sudo pacman -U " + pkg + "-" + VERSION + "-1-x86_64.pkg.tar.zst"),
    ];
  }

  function recommend(a) {
    if (!a.os) return null;
    if (a.os === "linux" && !a.distro) return null;
    if (!a.python || !a.use) return null;

    const server = a.use === "server";
    const noPython = a.python === "no";
    const r = { title: "", why: "", files: [], steps: [], notes: [] };

    // Intel Macs: no portable build yet
    if (a.os === "macos-intel" && noPython) {
      r.notes.push("There is no portable version for Intel Macs yet, so this is the Python version: you need to install Python first (step 1).");
    }
    const usePortable = noPython && a.os !== "macos-intel";

    if (usePortable && !server) {
      r.title = "bubbros-portable";
      r.why = "One program with the game server built in: start it and play, nothing else to install.";
      if (a.os === "windows") {
        r.files = ["bubbros-portable-x86_64-pc-windows-msvc.zip"];
        r.steps = [
          "Unzip the file (right-click, <em>Extract All</em>).",
          "Open the folder and double-click <code>bubbros-client.exe</code>. If Windows says it protected your PC, choose <em>More info</em>, then <em>Run anyway</em>.",
          "Choose a level set and press <em>Start</em>. Click on a dragon at the bottom, then press the four keys you want for right, left, jump and fire.",
        ];
      } else if (a.os === "macos-arm") {
        r.files = ["bubbros-portable-aarch64-apple-darwin.tar.gz"];
        r.steps = [
          "Double-click the file to unpack it.",
          "In the new folder, right-click <code>bubbros-client</code> and choose <em>Open</em> (needed the first time, because the program is not signed).",
          "Choose a level set and press <em>Start</em>. Click on a dragon at the bottom, then press your four keys: right, left, jump and fire.",
        ];
      } else if (a.os === "linux") {
        if (PKG[a.distro]) {
          const f = linuxPackage("bubbros-portable", a.distro);
          r.files = [f];
          r.steps = [
            "Install it from the folder you downloaded it to:" + code(PKG[a.distro].install(f)),
            "Start <em>Bub-n-Bros play alone</em> from your applications menu (or run <code>bubbros-client</code>).",
            "Choose a level set and press <em>Start</em>; click on a dragon and press your four keys.",
          ];
        } else if (a.distro === "arch") {
          r.steps = archSteps("bubbros-portable").concat([
            "Start <em>Bub-n-Bros play alone</em> from your applications menu (or run <code>bubbros-client</code>).",
          ]);
        } else {
          r.files = ["bubbros-portable-x86_64-unknown-linux-gnu.tar.gz"];
          r.steps = [
            "Unpack it and start the client:" + code("tar xzf bubbros-portable-x86_64-unknown-linux-gnu.tar.gz\ncd bubbros-portable-x86_64-unknown-linux-gnu\n./bubbros-client"),
            "Choose a level set and press <em>Start</em>; click on a dragon and press your four keys.",
          ];
          r.notes.push("Built on Ubuntu 24.04: it needs a distribution of 2024 or newer, with the usual X11, OpenGL and ALSA libraries.");
        }
      }
      r.notes.push("Several people can play on the same keyboard: each one clicks on a dragon and picks four keys.");
    } else if (usePortable && server) {
      r.title = "bubbros-server-portable";
      r.why = "The game server as one program with its own Python inside, for games that other computers join.";
      if (a.os === "windows") {
        r.files = ["bubbros-server-portable-x86_64-pc-windows-msvc.zip", "bubbros-portable-x86_64-pc-windows-msvc.zip"];
        r.steps = [
          "Unzip the server. In its folder, double-click <code>bubnbros-server.exe</code> to start a game with the standard levels; allow it through the firewall when Windows asks. (Advanced: <code>bubnbros-server.exe --web</code> in a command prompt opens the web launcher, to choose levels and options in your browser.)",
          "Every player, on this computer or another one on the network, runs the portable client (second file): <code>bubbros-client.exe</code> finds the server on the network by itself.",
        ];
      } else if (a.os === "macos-arm") {
        r.files = ["bubbros-server-portable-aarch64-apple-darwin.tar.gz", "bubbros-portable-aarch64-apple-darwin.tar.gz"];
        r.steps = [
          "Unpack the server and start it from a Terminal in its folder:" + code("./bubnbros-server           # the standard levels\n./bubnbros-server --web     # or: the web launcher, in your browser"),
          "Every player runs the portable client (second file): <code>bubbros-client</code> finds the server on the network by itself.",
        ];
      } else if (a.os === "linux") {
        if (PKG[a.distro]) {
          const f = linuxPackage("bubbros-server-portable", a.distro);
          r.files = [f];
          r.steps = [
            "Install it:" + code(PKG[a.distro].install(f)),
            "Start <em>Bub-n-Bros server</em> from your applications menu: your browser opens the web launcher, where you start a game.",
            "Players on this computer use <em>Join</em> in the launcher; players on other computers run the portable client (or any Bub's Brothers client), which finds the server on the network.",
          ];
        } else if (a.distro === "arch") {
          r.steps = archSteps("bubbros-server-portable").concat([
            "Start <em>Bub-n-Bros server</em> from your applications menu: your browser opens the web launcher.",
          ]);
        } else {
          r.files = ["bubbros-server-portable-x86_64-unknown-linux-gnu.tar.gz", "bubbros-portable-x86_64-unknown-linux-gnu.tar.gz"];
          r.steps = [
            "Unpack the server and start it:" + code("tar xzf bubbros-server-portable-x86_64-unknown-linux-gnu.tar.gz\ncd bubbros-server-portable-x86_64-unknown-linux-gnu\n./bubnbros-server           # the standard levels\n./bubnbros-server --web     # or: the web launcher, in your browser"),
            "Every player runs the portable client (second file), which finds the server on the network.",
          ];
        }
      }
      r.notes.push("Players' computers must be on the same network, and your firewall must let the game through.");
    } else {
      // the Python version
      r.title = "bubbros-python";
      r.why = server
        ? "The classic game for your own Python: its web launcher starts a game that other computers join."
        : "The classic game for your own Python: the web launcher starts a game, then you join it.";
      if (a.os === "windows") {
        r.files = ["bubbros-python-2.4.0.zip"];
        r.steps = [
          "Install Python 3 from <a href=\"https://www.python.org/downloads/\">python.org</a> (tick <em>Add python.exe to PATH</em>), then in a command prompt:" + code("py -m pip install pygame"),
          "Unzip the file and double-click <code>BubBob.py</code> (or run <code>py BubBob.py</code> in its folder). Your browser opens the web launcher.",
          server ? "Click <em>Start a new game</em>. Players on other computers join with any client, for example the portable one." :
                   "Click <em>Start a new game</em>, then join it; click on a dragon and press your four keys.",
        ];
      } else if (a.os === "macos-arm" || a.os === "macos-intel") {
        r.files = ["bubbros-python-2.4.0.tar.gz"];
        r.steps = [
          "Install Python 3 from <a href=\"https://www.python.org/downloads/\">python.org</a>, then in a Terminal:" + code("python3 -m pip install pygame"),
          "Unpack the file and start the web launcher:" + code("cd bubbros-python-2.4.0\npython3 BubBob.py"),
          server ? "Click <em>Start a new game</em> in your browser. Players on other computers join with any client." :
                   "Click <em>Start a new game</em>, then join it; click on a dragon and press your four keys.",
        ];
      } else if (a.os === "linux") {
        if (PKG[a.distro]) {
          const f = linuxPackage("bubbros-python", a.distro);
          r.files = [f];
          r.steps = [
            "Install it (this also installs Python and pygame):" + code(PKG[a.distro].install(f)),
            server ? "Start <em>Bub-n-Bros server</em> from your applications menu: your browser opens the web launcher, where you start a game that other computers join." :
                     "Start <em>Bub-n-Bros play alone</em> from your applications menu, or <em>Bub-n-Bros server</em> for the web launcher.",
          ];
          if (a.distro === "openmandriva") {
            r.notes.push("OpenMandriva has pygame (<code>python-pygame</code>) in its <em>unsupported</em> repository: enable it before installing.");
          }
        } else if (a.distro === "arch") {
          r.steps = archSteps("bubbros-python").concat([
            server ? "Start <em>Bub-n-Bros server</em> from your applications menu." : "Start <em>Bub-n-Bros play alone</em> from your applications menu.",
          ]);
        } else {
          r.files = ["bubbros-python-2.4.0.tar.gz"];
          r.steps = [
            "Install Python 3 and pygame from your distribution (for example <code>python3-pygame</code>).",
            "Unpack the file and start the web launcher:" + code("tar xzf bubbros-python-2.4.0.tar.gz\ncd bubbros-python-2.4.0\npython3 BubBob.py"),
            "Click <em>Start a new game</em> in your browser" + (server ? "; players on other computers join with any client." : ", then join it."),
          ];
        }
      }
    }
    return r;
  }

  // ---------------------------------------------------------------- display

  function render() {
    const a = answers();
    $("distro-step").hidden = a.os !== "linux";
    renumber();
    const r = recommend(a);
    const box = $("result");
    if (!r) {
      box.hidden = true;
      return;
    }
    let html = "<h3>" + (r.files.length ? "Your download: " : "Your package: ") + r.title + "</h3><p>" + r.why + "</p>";
    if (r.files.length) {
      html += '<div class="files">';
      for (const f of r.files) {
        const size = RELEASE_FILES[f];
        html += '<a class="file" href="' + fileUrl(f) + '"><span>⬇ ' + f + "</span><small>" +
                (size ? humanSize(size) : "") + "</small></a>";
      }
      html += "</div>";
    }
    html += "<ol>" + r.steps.map((s) => "<li>" + s + "</li>").join("") + "</ol>";
    for (const n of r.notes) html += '<p class="warn">' + n + "</p>";
    box.innerHTML = html;
    box.hidden = false;
  }

  function renumber() {
    let n = 1;
    for (const step of document.querySelectorAll(".step")) {
      if (step.hidden) continue;
      step.querySelector(".num").textContent = n++;
    }
  }

  function buildTable() {
    const body = $("files-body");
    const kinds = ["portable", "server-portable", "python", "debug"];
    const systems = ["windows", "macos-arm", "linux-generic", "ubuntu", "debian", "fedora", "openmandriva", "any"];
    const titles = { "portable": "bubbros-portable", "server-portable": "bubbros-server-portable",
                     "python": "bubbros-python", "debug": "Debugging files" };
    const names = Object.keys(RELEASE_FILES);
    for (const kind of kinds) {
      const group = names.filter((n) => describeFile(n).kind === kind).sort((x, y) =>
        (systems.indexOf(describeFile(x).system) - systems.indexOf(describeFile(y).system)) || x.localeCompare(y));
      if (!group.length) continue;
      const head = document.createElement("tr");
      head.className = "group";
      head.innerHTML = '<th colspan="4"><span>' + titles[kind] + "</span> " + KINDS[kind] + "</th>";
      body.appendChild(head);
      for (const name of group) {
        const d = describeFile(name);
        const tr = document.createElement("tr");
        if (kind === "debug") tr.className = "minor";
        tr.innerHTML =
          '<td class="file"><a href="' + fileUrl(name) + '">' + name + "</a></td>" +
          "<td>" + SYSTEMS[d.system] + "</td>" +
          "<td>" + d.format + "</td>" +
          '<td class="size">' + humanSize(RELEASE_FILES[name]) + "</td>";
        body.appendChild(tr);
      }
    }
  }

  // ---------------------------------------------------------------- start

  function init() {
    for (const el of document.querySelectorAll(".version")) el.textContent = VERSION;
    for (const el of document.querySelectorAll("a.release-page")) el.href = RELEASE_PAGE;

    const det = detectSystem();
    const setDetected = (os, note) => {
      if (os) $("os").value = os;
      $("detected").textContent = note
        ? "We think you are using " + note + ". Change it below if that is wrong."
        : "We could not tell which system you use: please choose it below.";
    };
    setDetected(det.os, det.note);
    if (det.distro) $("distro").value = det.distro;
    if (det.mac) {
      $("detected").textContent += " Intel Macs: choose \"macOS, Intel\".";
      refineMac((os, note) => { setDetected(os, note); render(); });
    }

    document.querySelectorAll("select, input").forEach((el) => el.addEventListener("change", render));
    buildTable();
    render();
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
