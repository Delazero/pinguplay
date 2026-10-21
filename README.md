# pinguplay

Now playing widget for OBS (Spotify) · Widget "tocando agora" para OBS (Spotify)

[Português](#português) · [English](#english)

---

## Português

Um card em formato de pílula que mostra a música tocando no Spotify, com um pinguim animado que dança enquanto a música toca e congela quando você pausa.

- **Sem login**: lê a música pelo controle de mídia do Windows. Não precisa de conta de desenvolvedor nem de token do Spotify.
- **Leve**: um `.exe` pequeno na bandeja do sistema. O widget usa só CSS para as animações.
- **Fácil de configurar**: página de configurações com prévia ao vivo. O OBS atualiza sozinho.
- 4 layouts: **Completo**, **Sem GIF**, **Minimalista** e **Personalizado** (escolha pinguim, capa, artista e barra).
- 6 temas (escuro por padrão, rosa, branco, cinza claro, lilás, menta) + cores personalizadas e opacidade do fundo.
- Animação trocável (GIF, WebP ou APNG).

### Requisitos

- Windows 10 ou 11
- App desktop do Spotify no mesmo PC da live
- OBS Studio (ou outro programa com fonte "Navegador")

### Como usar

1. Baixe o zip da página de [Releases](../../releases) e extraia numa pasta.
2. Abra o `pinguplay.exe`. Um ícone de disco cinza aparece na bandeja do Windows.
3. Clique com o botão direito no ícone → **Abrir configurações**.
4. No OBS: **Fontes → + → Navegador**, cole `http://localhost:7878/`. Largura = largura do card + 16; altura = 96 (ou 64 nos layouts sem imagem). A página de configurações mostra os valores certos.

O menu da bandeja também tem **Copiar URL do widget** e **Iniciar com o Windows**.

### Comportamento

| Situação | Widget |
| --- | --- |
| Tocando | Pinguim animado, barra de progresso andando |
| Pausado | Card continua; pinguim congela (sem pinguim: ⏸ sobre a capa; sem imagem: card translúcido) |
| Anúncio (Spotify free) | "Intervalo ♪" (pinguim congelado, se visível) |
| Spotify fechado | Card some com fade |

### Layouts

| Layout | Pinguim | Capa | Título | Artista | Barra |
| --- | --- | --- | --- | --- | --- |
| Completo | ✓ | ✓ | ✓ | ✓ | ✓ |
| Sem GIF | — | ✓ | ✓ | ✓ | ✓ |
| Minimalista | — | — | ✓ | ✓ | — |
| Personalizado | opcional | opcional | ✓ | opcional | opcional |

Sem pinguim e sem capa, o card vira uma pílula baixa com `Título · Artista` numa linha só.

### Trocar a animação

Na página de configurações, use **Enviar arquivo…** ou coloque o arquivo na pasta `assets/` ao lado do `.exe` e escolha na lista. Formatos: GIF, WebP, APNG (até 10 MB).

### Configuração manual

Tudo fica em `config.toml`, ao lado do `.exe`. Mudar `port` exige reiniciar o app.

### Compilar

```bash
cargo build --release
```

O executável fica em `target/release/pinguplay.exe`.

---

## English

A pill-shaped card that shows what's playing on Spotify, with an animated penguin that dances while music plays and freezes when you pause.

- **No login**: reads the track from Windows media controls. No Spotify developer account or token needed.
- **Lightweight**: a small tray `.exe`. The widget animates with CSS only.
- **Easy setup**: settings page with live preview. OBS updates by itself.
- 4 layouts: **Full**, **No GIF**, **Minimal** and **Custom** (pick penguin, cover, artist and progress bar).
- 6 themes (dark by default, pink, white, light gray, lilac, mint) + custom colors and background opacity.
- Swappable animation (GIF, WebP or APNG).

### Requirements

- Windows 10 or 11
- Spotify desktop app on the streaming PC
- OBS Studio (or any app with a "Browser" source)

### Usage

1. Download the zip from [Releases](../../releases) and extract it.
2. Run `pinguplay.exe`. A gray disc icon shows up in the Windows tray.
3. Right-click the icon → **Open settings**.
4. In OBS: **Sources → + → Browser**, paste `http://localhost:7878/`. Width = card width + 16; height = 96 (or 64 for image-less layouts). The settings page shows the exact values.

The tray menu also has **Copy widget URL** and **Start with Windows**.

### Behavior

| Situation | Widget |
| --- | --- |
| Playing | Animated penguin, moving progress bar |
| Paused | Card stays; penguin freezes (no penguin: ⏸ over the cover; no image: card dims) |
| Ad (Spotify free) | "Break ♪" (frozen penguin, if shown) |
| Spotify closed | Card fades out |

### Layouts

| Layout | Penguin | Cover | Title | Artist | Bar |
| --- | --- | --- | --- | --- | --- |
| Full | ✓ | ✓ | ✓ | ✓ | ✓ |
| No GIF | — | ✓ | ✓ | ✓ | ✓ |
| Minimal | — | — | ✓ | ✓ | — |
| Custom | optional | optional | ✓ | optional | optional |

With no penguin and no cover, the card becomes a short pill with `Title · Artist` on one line.

### Changing the animation

In settings, use **Upload file…**, or drop a file into the `assets/` folder next to the `.exe` and pick it from the list. Formats: GIF, WebP, APNG (up to 10 MB).

### Manual configuration

Everything lives in `config.toml` next to the `.exe`. Changing `port` requires restarting the app.

### Building

```bash
cargo build --release
```

The executable is at `target/release/pinguplay.exe`.

---

MIT License. Nunito font under SIL OFL 1.1.
