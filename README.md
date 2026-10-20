# pinguplay

Now playing widget for OBS (Spotify) · Widget "tocando agora" para OBS (Spotify)

[Português](#português) · [English](#english)

---

## Português

Um card em formato de pílula que mostra a música tocando no Spotify, com um pinguim animado que dança enquanto a música toca e congela quando você pausa.

- **Sem login**: lê a música pelo controle de mídia do Windows. Não precisa de conta de desenvolvedor nem de token do Spotify.
- **Leve**: um `.exe` pequeno na bandeja do sistema. O widget usa só CSS para as animações.
- **Fácil de configurar**: página de configurações com prévia ao vivo. O OBS atualiza sozinho.
- 6 temas (rosa, branco, cinza claro, escuro, lilás, menta) + cores personalizadas e opacidade do fundo.
- Animação trocável (GIF, WebP ou APNG).

### Requisitos

- Windows 10 ou 11
- App desktop do Spotify no mesmo PC da live
- OBS Studio (ou outro programa com fonte "Navegador")

### Como usar

1. Baixe o zip da página de [Releases](../../releases) e extraia numa pasta.
2. Abra o `pinguplay.exe`. Um ícone rosa aparece na bandeja do Windows.
3. Clique com o botão direito no ícone → **Abrir configurações**.
4. No OBS: **Fontes → + → Navegador**, cole `http://localhost:7878/`. Largura = largura do card + 16, altura = 96.

O menu da bandeja também tem **Copiar URL do widget** e **Iniciar com o Windows**.

### Comportamento

| Situação | Widget |
| --- | --- |
| Tocando | Pinguim animado, barra de progresso andando |
| Pausado | Card continua, pinguim congelado |
| Anúncio (Spotify free) | "Intervalo ♪" com pinguim congelado |
| Spotify fechado | Card some com fade |

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
- 6 themes (pink, white, light gray, dark, lilac, mint) + custom colors and background opacity.
- Swappable animation (GIF, WebP or APNG).

### Requirements

- Windows 10 or 11
- Spotify desktop app on the streaming PC
- OBS Studio (or any app with a "Browser" source)

### Usage

1. Download the zip from [Releases](../../releases) and extract it.
2. Run `pinguplay.exe`. A pink icon shows up in the Windows tray.
3. Right-click the icon → **Open settings**.
4. In OBS: **Sources → + → Browser**, paste `http://localhost:7878/`. Width = card width + 16, height = 96.

The tray menu also has **Copy widget URL** and **Start with Windows**.

### Behavior

| Situation | Widget |
| --- | --- |
| Playing | Animated penguin, moving progress bar |
| Paused | Card stays, penguin frozen |
| Ad (Spotify free) | "Break ♪" with frozen penguin |
| Spotify closed | Card fades out |

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
