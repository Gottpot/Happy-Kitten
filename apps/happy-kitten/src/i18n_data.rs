//! Translations of the user interface. The German text is the key (see `i18n.rs`).
//! Columns: English, Japanese, Korean, Spanish. Generated once, now edited by hand.

pub const ENTRIES: &[(&str, [&str; 4])] = &[
    (
        "Beim Export ist etwas schiefgelaufen",
        [
            "Something went wrong during export",
            "書き出し中に問題が発生しました",
            "내보내는 중 문제가 발생했어요",
            "Algo salió mal durante la exportación",
        ],
    ),
    (
        "Abgebrochen",
        [
            "Cancelled",
            "キャンセルしました",
            "취소되었어요",
            "Cancelado",
        ],
    ),
    (
        "Der Encoder antwortet nicht mehr",
        [
            "The encoder is no longer responding",
            "エンコーダーが応答しなくなりました",
            "인코더가 응답하지 않아요",
            "El codificador ha dejado de responder",
        ],
    ),
    (
        "Über Happy-Kitten",
        [
            "About Happy-Kitten",
            "Happy-Kittenについて",
            "Happy-Kitten 정보",
            "Acerca de Happy-Kitten",
        ],
    ),
    (
        "Erstellt von Gottpot",
        [
            "Created by Gottpot",
            "作者: Gottpot",
            "만든 사람: Gottpot",
            "Creado por Gottpot",
        ],
    ),
    (
        "Ein lokaler, werbefreier Videoschnitt für den Mac: ohne Cloud, ohne Konto, ohne Abo.",
        [
            "A local, ad-free video editor for the Mac: no cloud, no account, no subscription.",
            "Mac用のローカルで広告のない動画編集ソフト。クラウドもアカウントもサブスクも不要です。",
            "클라우드도 계정도 구독도 필요 없는, 광고 없는 Mac용 로컬 영상 편집기예요.",
            "Un editor de vídeo local y sin anuncios para Mac: sin nube, sin cuenta, sin suscripción.",
        ],
    ),
    (
        "Quelloffen unter MIT oder Apache-2.0.",
        [
            "Open source under MIT or Apache-2.0.",
            "MITまたはApache-2.0のオープンソースです。",
            "MIT 또는 Apache-2.0 라이선스의 오픈 소스예요.",
            "Código abierto bajo MIT o Apache-2.0.",
        ],
    ),
    (
        "Mit Dank an egui, wgpu, symphonia, cpal, fontdue, image und viele weitere Open-Source-Bibliotheken (siehe THIRD-PARTY-LICENSES.md).",
        [
            "With thanks to egui, wgpu, symphonia, cpal, fontdue, image and many more open-source libraries (see THIRD-PARTY-LICENSES.md).",
            "egui、wgpu、symphonia、cpal、fontdue、imageをはじめ、多くのオープンソースライブラリに感謝します（THIRD-PARTY-LICENSES.md参照）。",
            "egui, wgpu, symphonia, cpal, fontdue, image 등 많은 오픈 소스 라이브러리에 감사드려요 (THIRD-PARTY-LICENSES.md 참조).",
            "Gracias a egui, wgpu, symphonia, cpal, fontdue, image y muchas otras bibliotecas de código abierto (ver THIRD-PARTY-LICENSES.md).",
        ],
    ),
    ("Schön!", ["Nice!", "いいね！", "좋아요!", "¡Genial!"]),
    ("Titel", ["Title", "タイトル", "제목", "Título"]),
    ("Untertitel", ["Subtitle", "字幕", "자막", "Subtítulo"]),
    ("Lyrics", ["Lyrics", "歌詞", "가사", "Letra"]),
    (
        "Neon-Pink",
        ["Neon pink", "ネオンピンク", "네온 핑크", "Rosa neón"],
    ),
    ("Comic", ["Comic", "コミック", "코믹", "Cómic"]),
    (
        "Schreibmaschine",
        [
            "Typewriter",
            "タイプライター",
            "타자기",
            "Máquina de escribir",
        ],
    ),
    (
        "Es war einmal …",
        [
            "Once upon a time …",
            "むかしむかし…",
            "옛날 옛적에…",
            "Érase una vez…",
        ],
    ),
    (
        "Wölkchen",
        ["Little cloud", "ふわふわ雲", "구름송이", "Nubecita"],
    ),
    ("Strudel", ["Whirl", "うずまき", "소용돌이", "Remolino"]),
    ("Wellen", ["Waves", "波", "물결", "Ondas"]),
    (
        "Rückgängig (Cmd+Z)",
        [
            "Undo (Cmd+Z)",
            "元に戻す (Cmd+Z)",
            "실행 취소 (Cmd+Z)",
            "Deshacer (Cmd+Z)",
        ],
    ),
    (
        "Wiederholen (Cmd+Shift+Z)",
        [
            "Redo (Cmd+Shift+Z)",
            "やり直す (Cmd+Shift+Z)",
            "다시 실행 (Cmd+Shift+Z)",
            "Rehacer (Cmd+Shift+Z)",
        ],
    ),
    ("Teilen", ["Split", "分割", "나누기", "Dividir"]),
    (
        "Clip am Zeiger teilen (S)",
        [
            "Split clip at the playhead (S)",
            "再生ヘッドでクリップを分割 (S)",
            "재생 헤드에서 클립 나누기 (S)",
            "Dividir el clip en el cursor (S)",
        ],
    ),
    ("Löschen", ["Delete", "削除", "삭제", "Eliminar"]),
    (
        "Auswahl löschen (Entf), mit Shift Lücke schließen",
        [
            "Delete selection (Del), with Shift close the gap",
            "選択を削除 (Delete)、Shiftで隙間を詰める",
            "선택 삭제 (Delete), Shift를 누르면 빈틈을 메워요",
            "Eliminar la selección (Supr); con Mayús se cierra el hueco",
        ],
    ),
    ("Einrasten", ["Snap", "スナップ", "스냅", "Imán"]),
    (
        "Clips rasten an Kanten und am Zeiger ein",
        [
            "Clips snap to edges and to the playhead",
            "クリップは端や再生ヘッドに吸着します",
            "클립이 가장자리와 재생 헤드에 달라붙어요",
            "Los clips se ajustan a los bordes y al cursor",
        ],
    ),
    (
        "Videospur",
        [
            "Video track",
            "ビデオトラック",
            "비디오 트랙",
            "Pista de vídeo",
        ],
    ),
    (
        "Weitere Video-/Text-Spur oben auf den Stapel legen",
        [
            "Add another video/text track on top of the stack",
            "ビデオ／テキストトラックを一番上に追加",
            "비디오/텍스트 트랙을 맨 위에 추가해요",
            "Añadir otra pista de vídeo/texto encima",
        ],
    ),
    (
        "Audiospur",
        [
            "Audio track",
            "オーディオトラック",
            "오디오 트랙",
            "Pista de audio",
        ],
    ),
    (
        "Weitere Audio-Spur",
        [
            "Add another audio track",
            "オーディオトラックを追加",
            "오디오 트랙 추가",
            "Añadir otra pista de audio",
        ],
    ),
    ("Zoom", ["Zoom", "ズーム", "확대/축소", "Zoom"]),
    ("Dauer", ["Duration", "長さ", "길이", "Duración"]),
    ("Entfernen", ["Remove", "取り除く", "제거", "Quitar"]),
    (
        "{n} Clips ausgewählt",
        [
            "{n} clips selected",
            "{n}個のクリップを選択中",
            "클립 {n}개 선택됨",
            "{n} clips seleccionados",
        ],
    ),
    (
        "Ziehen = alle verschieben · Entf = alle löschen · Blenden-Tab: Blende für alle Schnittpunkte · Text-Tab: Stil für alle Texte",
        [
            "Drag = move all · Del = delete all · Transitions tab: transition at every cut · Text tab: style for all texts",
            "ドラッグ＝すべて移動 · Delete＝すべて削除 · 切り替えタブ: すべての切れ目に適用 · テキストタブ: すべてのテキストにスタイル適用",
            "드래그 = 모두 이동 · Delete = 모두 삭제 · 전환 탭: 모든 편집점에 전환 · 텍스트 탭: 모든 텍스트에 스타일",
            "Arrastrar = mover todo · Supr = borrar todo · Pestaña Transiciones: transición en cada corte · Pestaña Texto: estilo para todos los textos",
        ],
    ),
    (
        "Text: {first}",
        [
            "Text: {first}",
            "テキスト: {first}",
            "텍스트: {first}",
            "Texto: {first}",
        ],
    ),
    (
        "Ränder ziehen = kürzen · S = teilen",
        [
            "Drag edges = trim · S = split",
            "端をドラッグ＝短縮 · S＝分割",
            "가장자리 드래그 = 줄이기 · S = 나누기",
            "Arrastrar los bordes = recortar · S = dividir",
        ],
    ),
    (
        "Tipp: Ziehe Clips, Texte und Blenden aus dem Pool auf die Spuren, kürze sie an den Rändern und teile mit S.",
        [
            "Tip: Drag clips, texts and transitions from the pool onto the tracks, trim them at the edges and split with S.",
            "ヒント: クリップ・テキスト・切り替えをプールからトラックにドラッグし、端で短縮、Sで分割できます。",
            "팁: 풀에서 클립, 텍스트, 전환을 트랙으로 끌어다 놓고 가장자리로 줄이고 S로 나눠 보세요.",
            "Consejo: arrastra clips, textos y transiciones desde el panel a las pistas, recórtalos por los bordes y divide con S.",
        ],
    ),
    (
        "Spur löschen (wenn leer)",
        [
            "Delete track (if empty)",
            "トラックを削除（空の場合）",
            "트랙 삭제 (비어 있을 때)",
            "Eliminar pista (si está vacía)",
        ],
    ),
    (
        "Spur ein-/ausblenden",
        [
            "Show/hide track",
            "トラックの表示／非表示",
            "트랙 표시/숨기기",
            "Mostrar/ocultar pista",
        ],
    ),
    (
        "Spur stumm schalten",
        [
            "Mute track",
            "トラックをミュート",
            "트랙 음소거",
            "Silenciar pista",
        ],
    ),
    (
        "Am Zeiger teilen",
        [
            "Split at playhead",
            "再生ヘッドで分割",
            "재생 헤드에서 나누기",
            "Dividir en el cursor",
        ],
    ),
    (
        "Löschen und Lücke schließen",
        [
            "Delete and close gap",
            "削除して隙間を詰める",
            "삭제하고 빈틈 메우기",
            "Eliminar y cerrar el hueco",
        ],
    ),
    (
        "Blende entfernen",
        [
            "Remove transition",
            "切り替えを削除",
            "전환 제거",
            "Quitar transición",
        ],
    ),
    (
        "Als Vollbild-Bild verwenden",
        [
            "Use as full-screen picture",
            "全画面の画像として使う",
            "전체 화면 이미지로 사용",
            "Usar como imagen a pantalla completa",
        ],
    ),
    (
        "Als Grafik über das Video legen",
        [
            "Place over the video as a graphic",
            "グラフィックとして動画の上に重ねる",
            "그래픽으로 영상 위에 올리기",
            "Colocar sobre el vídeo como gráfico",
        ],
    ),
    (
        "Vorlage auf {n} Clips anwenden",
        [
            "Apply preset to {n} clips",
            "プリセットを{n}個のクリップに適用",
            "프리셋을 클립 {n}개에 적용",
            "Aplicar plantilla a {n} clips",
        ],
    ),
    (
        "Vorlage anwenden",
        [
            "Apply preset",
            "プリセットを適用",
            "프리셋 적용",
            "Aplicar plantilla",
        ],
    ),
    (
        "Setze den Zeiger auf den Clip, um ihn zu teilen",
        [
            "Put the playhead on the clip to split it",
            "分割するにはクリップの上に再生ヘッドを置いてください",
            "나누려면 재생 헤드를 클립 위에 놓아 주세요",
            "Coloca el cursor sobre el clip para dividirlo",
        ],
    ),
    (
        "MP4-Video",
        ["MP4 video", "MP4ビデオ", "MP4 영상", "Vídeo MP4"],
    ),
    (
        "Film exportieren",
        [
            "Export movie",
            "ムービーを書き出す",
            "영상 내보내기",
            "Exportar película",
        ],
    ),
    ("Format", ["Format", "フォーマット", "형식", "Formato"]),
    (
        "Bild anpassen",
        [
            "Fit picture",
            "画像のフィット",
            "화면 맞춤",
            "Ajustar imagen",
        ],
    ),
    (
        "Auflösung",
        ["Resolution", "解像度", "해상도", "Resolución"],
    ),
    (
        "Bildrate",
        [
            "Frame rate",
            "フレームレート",
            "프레임 속도",
            "Fotogramas por segundo",
        ],
    ),
    (
        "Wie das Projekt ({fps})",
        [
            "Same as project ({fps})",
            "プロジェクトと同じ ({fps})",
            "프로젝트와 동일 ({fps})",
            "Como el proyecto ({fps})",
        ],
    ),
    ("Qualität", ["Quality", "品質", "품질", "Calidad"]),
    (
        "Bitrate",
        ["Bitrate", "ビットレート", "비트레이트", "Tasa de bits"],
    ),
    (
        "selbst festlegen",
        [
            "set manually",
            "手動で設定",
            "직접 설정",
            "definir manualmente",
        ],
    ),
    (
        "automatisch ≈ {mbit} Mbit/s",
        [
            "automatic ≈ {mbit} Mbit/s",
            "自動 ≈ {mbit} Mbit/s",
            "자동 ≈ {mbit} Mbit/s",
            "automática ≈ {mbit} Mbit/s",
        ],
    ),
    ("Audio", ["Audio", "オーディオ", "오디오", "Audio"]),
    (
        "Audio-Bitrate",
        [
            "Audio bitrate",
            "オーディオビットレート",
            "오디오 비트레이트",
            "Tasa de bits del audio",
        ],
    ),
    ("Datei", ["File", "ファイル", "파일", "Archivo"]),
    (
        "MP4 · HEVC (kleiner)",
        [
            "MP4 · HEVC (smaller)",
            "MP4 · HEVC（小さめ）",
            "MP4 · HEVC (더 작음)",
            "MP4 · HEVC (más pequeño)",
        ],
    ),
    (
        "MP4 · H.264 (läuft überall)",
        [
            "MP4 · H.264 (plays everywhere)",
            "MP4 · H.264（どこでも再生可）",
            "MP4 · H.264 (어디서나 재생)",
            "MP4 · H.264 (se reproduce en todas partes)",
        ],
    ),
    (
        "{w}×{h} · {fps} Bilder/s · {secs} s · ca. {mb} MB",
        [
            "{w}×{h} · {fps} fps · {secs} s · approx. {mb} MB",
            "{w}×{h} · {fps} fps · {secs}秒 · 約{mb} MB",
            "{w}×{h} · {fps} fps · {secs}초 · 약 {mb} MB",
            "{w}×{h} · {fps} fps · {secs} s · aprox. {mb} MB",
        ],
    ),
    (
        "Speicherort wählen & los",
        [
            "Choose location & go",
            "保存先を選んで開始",
            "저장 위치 선택 후 시작",
            "Elegir ubicación y empezar",
        ],
    ),
    ("Abbrechen", ["Cancel", "キャンセル", "취소", "Cancelar"]),
    (
        "Export läuft",
        ["Exporting", "書き出し中", "내보내는 중", "Exportando"],
    ),
    (
        "Das Kätzchen rendert …",
        [
            "The kitten is rendering …",
            "子猫がレンダリング中…",
            "고양이가 렌더링 중이에요…",
            "El gatito está renderizando…",
        ],
    ),
    (
        "{secs} s vergangen",
        [
            "{secs} s elapsed",
            "経過 {secs}秒",
            "{secs}초 지남",
            "{secs} s transcurridos",
        ],
    ),
    (
        " · noch ca. {secs} s",
        [
            " · about {secs} s left",
            " · 残り約{secs}秒",
            " · 약 {secs}초 남음",
            " · quedan aprox. {secs} s",
        ],
    ),
    (
        "Export fertig ✓ {name}",
        [
            "Export finished ✓ {name}",
            "書き出し完了 ✓ {name}",
            "내보내기 완료 ✓ {name}",
            "Exportación terminada ✓ {name}",
        ],
    ),
    (
        "Die Timeline ist leer – zieh erst etwas hinein",
        [
            "The timeline is empty – drag something in first",
            "タイムラインが空です。先に何かをドラッグしてください",
            "타임라인이 비어 있어요. 먼저 무언가를 끌어다 놓아 주세요",
            "La línea de tiempo está vacía: arrastra algo primero",
        ],
    ),
    (
        "Der Ton wird noch geladen – gleich nochmal versuchen",
        [
            "The audio is still loading – try again in a moment",
            "音声を読み込み中です。少し待ってからもう一度お試しください",
            "소리를 아직 불러오는 중이에요. 잠시 후 다시 시도해 주세요",
            "El audio aún se está cargando: inténtalo de nuevo en un momento",
        ],
    ),
    (
        "Export:",
        ["Export:", "書き出し:", "내보내기:", "Exportación:"],
    ),
    (
        "Zieh ein Medium auf die Timeline",
        [
            "Drag a media file onto the timeline",
            "メディアをタイムラインにドラッグしてください",
            "미디어를 타임라인으로 끌어다 놓으세요",
            "Arrastra un archivo a la línea de tiempo",
        ],
    ),
    (
        "Bildformat",
        ["Aspect ratio", "画面比率", "화면 비율", "Formato de imagen"],
    ),
    (
        "Hochformat: Shorts, Reels, TikTok",
        [
            "Portrait: Shorts, Reels, TikTok",
            "縦向き: ショート、リール、TikTok",
            "세로 화면: 쇼츠, 릴스, 틱톡",
            "Vertical: Shorts, Reels, TikTok",
        ],
    ),
    (
        "Bildformat des Projekts",
        [
            "Aspect ratio of the project",
            "プロジェクトの画面比率",
            "프로젝트 화면 비율",
            "Formato de imagen del proyecto",
        ],
    ),
    (
        "Zum Anfang (Pos1)",
        [
            "To the start (Home)",
            "先頭へ (Home)",
            "처음으로 (Home)",
            "Al principio (Inicio)",
        ],
    ),
    (
        "Ein Bild zurück (←)",
        [
            "One frame back (←)",
            "1フレーム戻る (←)",
            "한 프레임 뒤로 (←)",
            "Un fotograma atrás (←)",
        ],
    ),
    (
        "Abspielen / Pause (Leertaste)",
        [
            "Play / pause (Space)",
            "再生／一時停止 (スペース)",
            "재생 / 일시정지 (스페이스)",
            "Reproducir / pausa (Espacio)",
        ],
    ),
    (
        "Ein Bild vor (→)",
        [
            "One frame forward (→)",
            "1フレーム進む (→)",
            "한 프레임 앞으로 (→)",
            "Un fotograma adelante (→)",
        ],
    ),
    (
        "Zum Ende (Ende)",
        [
            "To the end (End)",
            "末尾へ (End)",
            "끝으로 (End)",
            "Al final (Fin)",
        ],
    ),
    (
        "Grafiken wählen (PNG mit Transparenz funktioniert am besten)",
        [
            "Choose graphics (PNG with transparency works best)",
            "グラフィックを選択（透明PNGが最適です）",
            "그래픽 선택 (투명 PNG가 가장 좋아요)",
            "Elegir gráficos (PNG con transparencia funciona mejor)",
        ],
    ),
    ("Bilder", ["Images", "画像", "이미지", "Imágenes"]),
    (
        "Weitere Grafiken",
        [
            "More graphics",
            "さらにグラフィック",
            "그래픽 더 보기",
            "Más gráficos",
        ],
    ),
    (
        "Grafik (PNG)",
        [
            "Graphic (PNG)",
            "グラフィック (PNG)",
            "그래픽 (PNG)",
            "Gráfico (PNG)",
        ],
    ),
    (
        "Bild wählen und über das Video legen – Transparenz bleibt erhalten",
        [
            "Choose an image and place it over the video – transparency is kept",
            "画像を選んで動画の上に重ねます。透明部分はそのまま残ります",
            "이미지를 골라 영상 위에 올려요. 투명도는 그대로 유지돼요",
            "Elige una imagen y colócala sobre el vídeo: se conserva la transparencia",
        ],
    ),
    (
        "Karte auf eine Videospur ziehen oder ＋ anklicken. Bilder mit Transparenz werden automatisch zu Grafiken.",
        [
            "Drag the card onto a video track or click ＋. Images with transparency automatically become graphics.",
            "カードをビデオトラックにドラッグするか＋をクリック。透明な画像は自動でグラフィックになります。",
            "카드를 비디오 트랙으로 끌거나 ＋를 누르세요. 투명한 이미지는 자동으로 그래픽이 돼요.",
            "Arrastra la tarjeta a una pista de vídeo o pulsa ＋. Las imágenes con transparencia se convierten automáticamente en gráficos.",
        ],
    ),
    (
        "Noch keine Bilder im Projekt. Importiere ein PNG oder wähle „Grafik (PNG)“.",
        [
            "No images in the project yet. Import a PNG or choose “Graphic (PNG)”.",
            "プロジェクトに画像がまだありません。PNGをインポートするか「グラフィック (PNG)」を選んでください。",
            "프로젝트에 아직 이미지가 없어요. PNG를 가져오거나 '그래픽 (PNG)'을 선택하세요.",
            "Aún no hay imágenes en el proyecto. Importa un PNG o elige «Gráfico (PNG)».",
        ],
    ),
    (
        "Auf eine Videospur ziehen: liegt als Grafik über dem Bild",
        [
            "Drag onto a video track: it lies over the picture as a graphic",
            "ビデオトラックにドラッグ: グラフィックとして映像の上に重なります",
            "비디오 트랙으로 끌기: 그래픽으로 화면 위에 올라가요",
            "Arrástralo a una pista de vídeo: queda sobre la imagen como gráfico",
        ],
    ),
    (
        "Grafik bearbeiten: {name}",
        [
            "Edit graphic: {name}",
            "グラフィックを編集: {name}",
            "그래픽 편집: {name}",
            "Editar gráfico: {name}",
        ],
    ),
    (
        "{n} Grafiken bearbeiten",
        [
            "Edit {n} graphics",
            "{n}個のグラフィックを編集",
            "그래픽 {n}개 편집",
            "Editar {n} gráficos",
        ],
    ),
    (
        "Änderungen gelten für alle gewählten Grafiken.",
        [
            "Changes apply to all selected graphics.",
            "変更は選択したすべてのグラフィックに反映されます。",
            "변경 사항은 선택한 모든 그래픽에 적용돼요.",
            "Los cambios se aplican a todos los gráficos seleccionados.",
        ],
    ),
    ("Größe", ["Size", "サイズ", "크기", "Tamaño"]),
    ("Position", ["Position", "位置", "위치", "Posición"]),
    ("Drehung", ["Rotation", "回転", "회전", "Rotación"]),
    ("Deckkraft", ["Opacity", "不透明度", "불투명도", "Opacidad"]),
    (
        "Ein- und Ausblenden",
        [
            "Fade in and out",
            "フェードイン／アウト",
            "페이드 인/아웃",
            "Aparecer y desaparecer",
        ],
    ),
    (
        "Einblenden",
        ["Fade in", "フェードイン", "페이드 인", "Aparecer"],
    ),
    (
        "Ausblenden",
        ["Fade out", "フェードアウト", "페이드 아웃", "Desaparecer"],
    ),
    (
        "Zurücksetzen",
        ["Reset", "リセット", "초기화", "Restablecer"],
    ),
    (
        "Größe, Position, Drehung und Deckkraft auf Standard",
        [
            "Size, position, rotation and opacity back to default",
            "サイズ・位置・回転・不透明度を初期値に戻します",
            "크기, 위치, 회전, 불투명도를 기본값으로 되돌려요",
            "Tamaño, posición, rotación y opacidad vuelven a los valores predeterminados",
        ],
    ),
    (
        "Als Vollbild-Bild",
        [
            "As full-screen picture",
            "全画面の画像にする",
            "전체 화면 이미지로",
            "Como imagen a pantalla completa",
        ],
    ),
    (
        "Nicht mehr über das Video legen, sondern als Bild füllen",
        [
            "No longer overlay the video, fill the frame as a picture instead",
            "動画に重ねず、画面いっぱいの画像として表示します",
            "영상 위에 올리지 않고 화면을 채우는 이미지로 써요",
            "Dejar de superponer al vídeo y llenar el encuadre como imagen",
        ],
    ),
    (
        "Das ist keine Bilddatei",
        [
            "This is not an image file",
            "これは画像ファイルではありません",
            "이미지 파일이 아니에요",
            "Esto no es un archivo de imagen",
        ],
    ),
    (
        "Dateien importieren (mehrere möglich)",
        [
            "Import files (multiple possible)",
            "ファイルをインポート（複数可）",
            "파일 가져오기 (여러 개 가능)",
            "Importar archivos (varios a la vez)",
        ],
    ),
    (
        "Alle unterstützten Dateien",
        [
            "All supported files",
            "対応するすべてのファイル",
            "지원되는 모든 파일",
            "Todos los archivos compatibles",
        ],
    ),
    (
        "Video und Bilder",
        [
            "Video and images",
            "動画と画像",
            "영상과 이미지",
            "Vídeo e imágenes",
        ],
    ),
    (
        "Ordner importieren",
        [
            "Import folder",
            "フォルダーをインポート",
            "폴더 가져오기",
            "Importar carpeta",
        ],
    ),
    (
        "Untertitel oder Lyrics importieren",
        [
            "Import subtitles or lyrics",
            "字幕または歌詞をインポート",
            "자막 또는 가사 가져오기",
            "Importar subtítulos o letras",
        ],
    ),
    (
        "Untertitel und Lyrics",
        [
            "Subtitles and lyrics",
            "字幕と歌詞",
            "자막과 가사",
            "Subtítulos y letras",
        ],
    ),
    (
        "{n} Datei(en) in die Bibliothek importiert",
        [
            "{n} file(s) imported into the library",
            "{n}個のファイルをライブラリにインポートしました",
            "파일 {n}개를 라이브러리로 가져왔어요",
            "{n} archivo(s) importados a la biblioteca",
        ],
    ),
    (
        "{n} Datei(en) ins Projekt importiert",
        [
            "{n} file(s) imported into the project",
            "{n}個のファイルをプロジェクトにインポートしました",
            "파일 {n}개를 프로젝트로 가져왔어요",
            "{n} archivo(s) importados al proyecto",
        ],
    ),
    (
        "{n} Textzeilen",
        [
            "{n} lines of text",
            "{n}行のテキスト",
            "텍스트 {n}줄",
            "{n} líneas de texto",
        ],
    ),
    (
        " · {n} nicht lesbar",
        [
            " · {n} unreadable",
            " · {n}個は読み込めません",
            " · {n}개는 읽을 수 없어요",
            " · {n} ilegibles",
        ],
    ),
    (
        "In {name} wurde kein Text gefunden",
        [
            "No text was found in {name}",
            "{name} にテキストが見つかりませんでした",
            "{name}에서 텍스트를 찾지 못했어요",
            "No se encontró texto en {name}",
        ],
    ),
    ("Text", ["Text", "テキスト", "텍스트", "Texto"]),
    (
        "{name} fehlt – die Datei wurde verschoben oder gelöscht",
        [
            "{name} is missing – the file was moved or deleted",
            "{name} が見つかりません。ファイルが移動または削除されました",
            "{name}을(를) 찾을 수 없어요. 파일이 이동되었거나 삭제되었어요",
            "Falta {name}: el archivo se movió o se eliminó",
        ],
    ),
    (
        "Bibliothek konnte nicht gespeichert werden:",
        [
            "The library could not be saved:",
            "ライブラリを保存できませんでした:",
            "라이브러리를 저장할 수 없었어요:",
            "No se pudo guardar la biblioteca:",
        ],
    ),
    ("Dateien", ["Files", "ファイル", "파일", "Archivos"]),
    ("Grafik", ["Graphic", "グラフィック", "그래픽", "Gráfico"]),
    (
        "Blenden",
        ["Transitions", "切り替え", "전환", "Transiciones"],
    ),
    (
        "Blende zwischen zwei Clips ziehen.",
        [
            "Drag a transition between two clips.",
            "切り替えを2つのクリップの間にドラッグします。",
            "전환을 두 클립 사이로 끌어다 놓으세요.",
            "Arrastra una transición entre dos clips.",
        ],
    ),
    (
        "Eigene Dauer",
        [
            "Custom duration",
            "長さを指定",
            "직접 길이 설정",
            "Duración propia",
        ],
    ),
    (
        "jede Blende mit ihrer eigenen Länge",
        [
            "each transition with its own length",
            "切り替えごとに個別の長さにします",
            "전환마다 자기만의 길이를 사용해요",
            "cada transición con su propia duración",
        ],
    ),
    (
        "Tipp: Clips mit dem Rahmen markieren, Blende anklicken = an allen Schnittpunkten.",
        [
            "Tip: select clips with the marquee, click a transition = at every cut.",
            "ヒント: 枠でクリップを選び、切り替えをクリックするとすべての切れ目に入ります。",
            "팁: 선택 상자로 클립을 고르고 전환을 누르면 모든 편집점에 들어가요.",
            "Consejo: selecciona clips con el recuadro y pulsa una transición para ponerla en todos los cortes.",
        ],
    ),
    (
        "{n} Clip(s) gewählt: Klick auf eine Blende setzt sie an allen Schnittpunkten der Auswahl.",
        [
            "{n} clip(s) selected: clicking a transition places it at every cut of the selection.",
            "{n}個のクリップを選択中: 切り替えをクリックすると、選択範囲のすべての切れ目に入ります。",
            "클립 {n}개 선택됨: 전환을 누르면 선택한 모든 편집점에 들어가요.",
            "{n} clip(s) seleccionados: al pulsar una transición se coloca en todos los cortes de la selección.",
        ],
    ),
    (
        "Katzen-Blenden",
        [
            "Kitten transitions",
            "猫の切り替え",
            "고양이 전환",
            "Transiciones gatunas",
        ],
    ),
    (
        "Klassische Blenden",
        [
            "Classic transitions",
            "クラシックな切り替え",
            "클래식 전환",
            "Transiciones clásicas",
        ],
    ),
    (
        "Ziehen und zwischen zwei Clips fallen lassen – oder anklicken, um sie an allen Schnittpunkten der Auswahl einzufügen",
        [
            "Drag and drop between two clips – or click to insert at every cut of the selection",
            "ドラッグして2つのクリップの間にドロップ、またはクリックで選択範囲のすべての切れ目に挿入",
            "끌어서 두 클립 사이에 놓거나, 클릭해서 선택한 모든 편집점에 넣으세요",
            "Arrástrala y suéltala entre dos clips, o haz clic para insertarla en todos los cortes de la selección",
        ],
    ),
    (
        "Vorlagen",
        ["Presets", "プリセット", "프리셋", "Plantillas"],
    ),
    (
        "Neuen Text am Zeiger einfügen",
        [
            "Insert new text at the playhead",
            "再生ヘッドの位置に新しいテキストを挿入",
            "재생 헤드 위치에 새 텍스트 넣기",
            "Insertar texto nuevo en el cursor",
        ],
    ),
    (
        "Untertitel oder Lyrics importieren: jede Zeile wird ein Text-Clip",
        [
            "Import subtitles or lyrics: every line becomes a text clip",
            "字幕や歌詞をインポート: 1行ごとにテキストクリップになります",
            "자막이나 가사 가져오기: 한 줄이 텍스트 클립 하나가 돼요",
            "Importar subtítulos o letras: cada línea se convierte en un clip de texto",
        ],
    ),
    (
        "Vorlage auf eine Spur ziehen oder ＋ anklicken.",
        [
            "Drag a preset onto a track or click ＋.",
            "プリセットをトラックにドラッグするか＋をクリックします。",
            "프리셋을 트랙으로 끌거나 ＋를 누르세요.",
            "Arrastra una plantilla a una pista o pulsa ＋.",
        ],
    ),
    (
        "Meine Vorlagen (global)",
        [
            "My presets (global)",
            "マイプリセット（グローバル）",
            "내 프리셋 (전역)",
            "Mis plantillas (globales)",
        ],
    ),
    (
        "Wähle einen Text-Clip in der Timeline, um Schrift, Farben und Effekte zu ändern.",
        [
            "Select a text clip in the timeline to change font, colours and effects.",
            "タイムラインでテキストクリップを選ぶと、フォント・色・エフェクトを変更できます。",
            "타임라인에서 텍스트 클립을 선택하면 글꼴, 색, 효과를 바꿀 수 있어요.",
            "Selecciona un clip de texto en la línea de tiempo para cambiar la fuente, los colores y los efectos.",
        ],
    ),
    (
        "Auf {n} gewählte Texte anwenden",
        [
            "Apply to {n} selected texts",
            "選択した{n}個のテキストに適用",
            "선택한 텍스트 {n}개에 적용",
            "Aplicar a los {n} textos seleccionados",
        ],
    ),
    (
        "Auf den gewählten Text anwenden",
        [
            "Apply to the selected text",
            "選択したテキストに適用",
            "선택한 텍스트에 적용",
            "Aplicar al texto seleccionado",
        ],
    ),
    (
        "Neuen Text mit dieser Vorlage einfügen",
        [
            "Insert new text with this preset",
            "このプリセットで新しいテキストを挿入",
            "이 프리셋으로 새 텍스트 넣기",
            "Insertar texto nuevo con esta plantilla",
        ],
    ),
    (
        "Name der Vorlage",
        [
            "Preset name",
            "プリセット名",
            "프리셋 이름",
            "Nombre de la plantilla",
        ],
    ),
    (
        "Vorlage löschen",
        [
            "Delete preset",
            "プリセットを削除",
            "프리셋 삭제",
            "Eliminar plantilla",
        ],
    ),
    (
        "Vorlage wählen …",
        [
            "Choose a preset …",
            "プリセットを選ぶ…",
            "프리셋 선택…",
            "Elegir plantilla…",
        ],
    ),
    (
        "Auf {n} Texte anwenden",
        [
            "Apply to {n} texts",
            "{n}個のテキストに適用",
            "텍스트 {n}개에 적용",
            "Aplicar a {n} textos",
        ],
    ),
    ("Anwenden", ["Apply", "適用", "적용", "Aplicar"]),
    (
        "Überträgt die Vorlage auf alle gewählten Texte (Wortlaut und, ohne Haken unten, Position bleiben)",
        [
            "Applies the preset to all selected texts (wording stays, and so does position unless the box below is ticked)",
            "プリセットを選択したすべてのテキストに適用します（文言は変わらず、下のチェックがなければ位置も変わりません）",
            "프리셋을 선택한 모든 텍스트에 적용해요 (문구는 그대로, 아래 체크가 없으면 위치도 그대로)",
            "Aplica la plantilla a todos los textos seleccionados (el texto se mantiene y, sin la casilla de abajo, la posición también)",
        ],
    ),
    (
        "Position der Vorlage mitnehmen",
        [
            "Take the preset's position along",
            "プリセットの位置も反映",
            "프리셋의 위치도 가져오기",
            "Incluir la posición de la plantilla",
        ],
    ),
    (
        "Vorlage auf {n} Text(e) angewendet",
        [
            "Preset applied to {n} text(s)",
            "プリセットを{n}個のテキストに適用しました",
            "프리셋을 텍스트 {n}개에 적용했어요",
            "Plantilla aplicada a {n} texto(s)",
        ],
    ),
    (
        "Text bearbeiten",
        ["Edit text", "テキストを編集", "텍스트 편집", "Editar texto"],
    ),
    (
        "{n} Texte bearbeiten",
        [
            "Edit {n} texts",
            "{n}個のテキストを編集",
            "텍스트 {n}개 편집",
            "Editar {n} textos",
        ],
    ),
    (
        "Schrift, Farben, Position und Effekte gelten für alle gewählten Texte, der Wortlaut nur für den ersten.",
        [
            "Font, colours, position and effects apply to all selected texts, the wording only to the first.",
            "フォント・色・位置・エフェクトは選択したすべてのテキストに、文言は最初の1つだけに反映されます。",
            "글꼴, 색, 위치, 효과는 선택한 모든 텍스트에 적용되고 문구는 첫 번째에만 적용돼요.",
            "La fuente, los colores, la posición y los efectos se aplican a todos los textos seleccionados; el texto, solo al primero.",
        ],
    ),
    (
        "Dein Text – jede Zeile bleibt eine Zeile",
        [
            "Your text – every line stays a line",
            "テキストを入力 – 改行はそのまま保たれます",
            "텍스트를 입력하세요 – 줄바꿈은 그대로 유지돼요",
            "Tu texto: cada línea sigue siendo una línea",
        ],
    ),
    ("Schrift", ["Font", "フォント", "글꼴", "Fuente"]),
    (
        "{label} (nicht installiert)",
        [
            "{label} (not installed)",
            "{label}（未インストール）",
            "{label} (설치되지 않음)",
            "{label} (no instalada)",
        ],
    ),
    ("Farbe", ["Colour", "色", "색상", "Color"]),
    ("Kontur", ["Outline", "縁取り", "윤곽선", "Contorno"]),
    ("Hintergrund", ["Background", "背景", "배경", "Fondo"]),
    ("Schatten", ["Shadow", "影", "그림자", "Sombra"]),
    ("Ausrichtung", ["Alignment", "配置", "정렬", "Alineación"]),
    ("Links", ["Left", "左", "왼쪽", "Izquierda"]),
    ("Mitte", ["Centre", "中央", "가운데", "Centro"]),
    ("Rechts", ["Right", "右", "오른쪽", "Derecha"]),
    (
        "Zeilenabstand",
        ["Line spacing", "行間", "줄 간격", "Interlineado"],
    ),
    (
        "Als Vorlage speichern",
        [
            "Save as preset",
            "プリセットとして保存",
            "프리셋으로 저장",
            "Guardar como plantilla",
        ],
    ),
    (
        "Merkt sich den Stil in der globalen Bibliothek, für alle Projekte",
        [
            "Remembers the style in the global library, for all projects",
            "スタイルをグローバルライブラリに保存し、すべてのプロジェクトで使えます",
            "스타일을 전역 라이브러리에 기억해 모든 프로젝트에서 쓸 수 있어요",
            "Guarda el estilo en la biblioteca global, para todos los proyectos",
        ],
    ),
    (
        "Mein Stil {n}",
        [
            "My style {n}",
            "マイスタイル {n}",
            "내 스타일 {n}",
            "Mi estilo {n}",
        ],
    ),
    (
        "Stil für alle Texte der Spur",
        [
            "Style for all texts on the track",
            "トラック内のすべてのテキストにスタイルを適用",
            "트랙의 모든 텍스트에 스타일 적용",
            "Estilo para todos los textos de la pista",
        ],
    ),
    (
        "Überträgt Schrift, Farben, Position und Effekte auf jeden Text dieser Spur",
        [
            "Applies font, colours, position and effects to every text on this track",
            "フォント・色・位置・エフェクトをこのトラックのすべてのテキストに反映します",
            "글꼴, 색, 위치, 효과를 이 트랙의 모든 텍스트에 적용해요",
            "Aplica fuente, colores, posición y efectos a cada texto de esta pista",
        ],
    ),
    (
        "Zeilen bunt färben",
        [
            "Colour lines in pastel",
            "行をカラフルに色付け",
            "줄마다 알록달록 색칠하기",
            "Colorear las líneas",
        ],
    ),
    (
        "Jeder Text der Spur bekommt der Reihe nach eine Pastellfarbe",
        [
            "Every text on the track gets a pastel colour in turn",
            "トラックの各テキストに順番にパステルカラーを付けます",
            "트랙의 각 텍스트가 차례로 파스텔 색을 받아요",
            "Cada texto de la pista recibe por turnos un color pastel",
        ],
    ),
    (
        "Wähle zuerst Text-Clips aus",
        [
            "Select text clips first",
            "先にテキストクリップを選択してください",
            "먼저 텍스트 클립을 선택해 주세요",
            "Selecciona primero clips de texto",
        ],
    ),
    (
        "Vorlage gespeichert – sie liegt oben unter „Meine Vorlagen“",
        [
            "Preset saved – you'll find it above under “My presets”",
            "プリセットを保存しました。上の「マイプリセット」にあります",
            "프리셋을 저장했어요. 위의 '내 프리셋'에서 찾을 수 있어요",
            "Plantilla guardada: la encontrarás arriba en «Mis plantillas»",
        ],
    ),
    (
        "Ungespeicherte Änderungen",
        [
            "Unsaved changes",
            "未保存の変更",
            "저장하지 않은 변경 사항",
            "Cambios sin guardar",
        ],
    ),
    (
        "Das Projekt hat Änderungen, die noch nicht gespeichert sind. Trotzdem fortfahren?",
        [
            "The project has changes that have not been saved yet. Continue anyway?",
            "プロジェクトにまだ保存されていない変更があります。このまま続けますか？",
            "프로젝트에 아직 저장하지 않은 변경 사항이 있어요. 그래도 계속할까요?",
            "El proyecto tiene cambios sin guardar. ¿Continuar de todos modos?",
        ],
    ),
    ("Speichern", ["Save", "保存", "저장", "Guardar"]),
    ("Verwerfen", ["Discard", "破棄", "버리기", "Descartar"]),
    (
        "Happy-Kitten-Projekt",
        [
            "Happy-Kitten project",
            "Happy-Kittenプロジェクト",
            "Happy-Kitten 프로젝트",
            "Proyecto de Happy-Kitten",
        ],
    ),
    (
        "{name} an {n} Schnittpunkt(en) eingefügt",
        [
            "{name} inserted at {n} cut(s)",
            "{name} を{n}か所の切れ目に挿入しました",
            "{name}을(를) 편집점 {n}곳에 넣었어요",
            "{name} insertada en {n} corte(s)",
        ],
    ),
    (
        "{name} eingefügt",
        [
            "{name} inserted",
            "{name} を挿入しました",
            "{name}을(를) 넣었어요",
            "{name} insertada",
        ],
    ),
    ("Neu", ["New", "新規", "새로 만들기", "Nuevo"]),
    ("Öffnen", ["Open", "開く", "열기", "Abrir"]),
    (
        "Exportieren",
        ["Export", "書き出す", "내보내기", "Exportar"],
    ),
    (
        "Film als MP4 speichern",
        [
            "Save the movie as MP4",
            "ムービーをMP4で保存",
            "영상을 MP4로 저장",
            "Guardar la película como MP4",
        ],
    ),
    (
        "Standbild",
        ["Still frame", "静止画", "정지 화면", "Fotograma"],
    ),
    (
        "Aktuelles Vorschaubild als PNG speichern",
        [
            "Save the current preview frame as PNG",
            "現在のプレビュー画像をPNGで保存",
            "현재 미리보기 화면을 PNG로 저장",
            "Guardar el fotograma actual de la vista previa como PNG",
        ],
    ),
    ("Import", ["Import", "インポート", "가져오기", "Importar"]),
    (
        "Nach dem letzten Absturz wiederhergestellt – bitte speichern",
        [
            "Restored after the last crash – please save",
            "前回のクラッシュから復元しました。保存してください",
            "지난 비정상 종료 후 복구했어요. 저장해 주세요",
            "Restaurado tras el último cierre inesperado: guarda el proyecto",
        ],
    ),
    (
        "Neues Projekt angelegt",
        [
            "New project created",
            "新しいプロジェクトを作成しました",
            "새 프로젝트를 만들었어요",
            "Proyecto nuevo creado",
        ],
    ),
    (
        "Projekt geöffnet",
        [
            "Project opened",
            "プロジェクトを開きました",
            "프로젝트를 열었어요",
            "Proyecto abierto",
        ],
    ),
    (
        "Öffnen fehlgeschlagen:",
        [
            "Opening failed:",
            "開けませんでした:",
            "열기에 실패했어요:",
            "No se pudo abrir:",
        ],
    ),
    (
        "Gespeichert ✓",
        ["Saved ✓", "保存しました ✓", "저장했어요 ✓", "Guardado ✓"],
    ),
    (
        "Speichern fehlgeschlagen:",
        [
            "Saving failed:",
            "保存できませんでした:",
            "저장에 실패했어요:",
            "No se pudo guardar:",
        ],
    ),
    (
        "Standbild gespeichert ✓",
        [
            "Still frame saved ✓",
            "静止画を保存しました ✓",
            "정지 화면을 저장했어요 ✓",
            "Fotograma guardado ✓",
        ],
    ),
    (
        "Standbild fehlgeschlagen:",
        [
            "Still frame failed:",
            "静止画の保存に失敗しました:",
            "정지 화면 저장에 실패했어요:",
            "Falló el fotograma:",
        ],
    ),
    (
        "Setze den Zeiger auf einen Clip, um ihn zu teilen",
        [
            "Put the playhead on a clip to split it",
            "分割するにはクリップの上に再生ヘッドを置いてください",
            "나누려면 재생 헤드를 클립 위에 놓아 주세요",
            "Coloca el cursor sobre un clip para dividirlo",
        ],
    ),
    (
        "Wähle erst Clips aus, oder zieh die Blende auf einen Schnittpunkt",
        [
            "Select clips first, or drag the transition onto a cut",
            "先にクリップを選ぶか、切り替えを切れ目にドラッグしてください",
            "먼저 클립을 선택하거나 전환을 편집점으로 끌어다 놓으세요",
            "Selecciona clips primero o arrastra la transición a un corte",
        ],
    ),
    (
        "Zwischen den gewählten Clips liegt kein Schnittpunkt",
        [
            "There is no cut between the selected clips",
            "選択したクリップの間に切れ目がありません",
            "선택한 클립 사이에 편집점이 없어요",
            "No hay ningún corte entre los clips seleccionados",
        ],
    ),
    ("Video", ["Video", "ビデオ", "비디오", "Vídeo"]),
    ("Bild", ["Image", "画像", "이미지", "Imagen"]),
    ("Ton", ["Sound", "音声", "소리", "Sonido"]),
    (
        "Mehrere Dateien auf einmal wählen (⌘I)",
        [
            "Choose several files at once (⌘I)",
            "複数のファイルを一度に選択 (⌘I)",
            "여러 파일을 한 번에 선택 (⌘I)",
            "Elegir varios archivos a la vez (⌘I)",
        ],
    ),
    ("Ordner", ["Folder", "フォルダー", "폴더", "Carpeta"]),
    (
        "Alle Videos, Bilder, Audios und Untertitel eines Ordners",
        [
            "All videos, images, audio files and subtitles of a folder",
            "フォルダー内のすべての動画・画像・音声・字幕",
            "폴더의 모든 영상, 이미지, 오디오, 자막",
            "Todos los vídeos, imágenes, audios y subtítulos de una carpeta",
        ],
    ),
    (
        "{n} von {total}",
        [
            "{n} of {total}",
            "{n} / {total}",
            "{total}개 중 {n}개",
            "{n} de {total}",
        ],
    ),
    (
        "Dieses Projekt",
        [
            "This project",
            "このプロジェクト",
            "이 프로젝트",
            "Este proyecto",
        ],
    ),
    (
        "Dateien, die nur zu diesem Projekt gehören. Goldener Haken = schon auf der Timeline verwendet",
        [
            "Files that belong to this project only. Golden tick = already used on the timeline",
            "このプロジェクト専用のファイル。金色のチェック＝タイムラインで使用済み",
            "이 프로젝트에만 속한 파일이에요. 금색 체크 = 타임라인에서 이미 사용됨",
            "Archivos que solo pertenecen a este proyecto. Marca dorada = ya usado en la línea de tiempo",
        ],
    ),
    ("Global", ["Global", "グローバル", "전역", "Global"]),
    (
        "Bibliothek für alle Projekte: Dateien bleiben, wo sie sind, und sind überall verfügbar",
        [
            "Library for all projects: files stay where they are and are available everywhere",
            "すべてのプロジェクトで使えるライブラリ: ファイルは元の場所に残り、どこからでも使えます",
            "모든 프로젝트용 라이브러리: 파일은 있던 자리에 두고 어디서나 쓸 수 있어요",
            "Biblioteca para todos los proyectos: los archivos se quedan donde están y están disponibles en todas partes",
        ],
    ),
    ("Suchen …", ["Search …", "検索…", "검색…", "Buscar…"]),
    (
        "Suche leeren",
        [
            "Clear search",
            "検索をクリア",
            "검색 지우기",
            "Borrar búsqueda",
        ],
    ),
    ("Kacheln", ["Tiles", "タイル", "타일", "Mosaicos"]),
    (
        "Liste mit ganzem Namen",
        [
            "List with full names",
            "完全な名前のリスト",
            "전체 이름 목록",
            "Lista con nombres completos",
        ],
    ),
    ("Tabelle", ["Table", "表", "표", "Tabla"]),
    (
        "Alle Arten",
        ["All types", "すべての種類", "모든 종류", "Todos los tipos"],
    ),
    ("Videos", ["Videos", "ビデオ", "비디오", "Vídeos"]),
    ("Alles", ["Everything", "すべて", "전체", "Todo"]),
    ("Verwendet", ["Used", "使用済み", "사용됨", "Usado"]),
    (
        "Noch frei",
        ["Not used yet", "未使用", "아직 안 씀", "Sin usar"],
    ),
    ("Reihenfolge", ["Order", "並び順", "정렬", "Orden"]),
    ("Name", ["Name", "名前", "이름", "Nombre"]),
    ("Art", ["Type", "種類", "종류", "Tipo"]),
    (
        "Absteigend",
        ["Descending", "降順", "내림차순", "Descendente"],
    ),
    (
        "Aufsteigend",
        ["Ascending", "昇順", "오름차순", "Ascendente"],
    ),
    (
        "Nichts passt zu Suche und Filtern.",
        [
            "Nothing matches the search and filters.",
            "検索とフィルターに一致するものはありません。",
            "검색과 필터에 맞는 항목이 없어요.",
            "Nada coincide con la búsqueda y los filtros.",
        ],
    ),
    (
        "Noch nichts da – importiere Dateien oder einen ganzen Ordner, oder zieh sie ins Fenster.",
        [
            "Nothing here yet – import files or a whole folder, or drag them into the window.",
            "まだ何もありません。ファイルやフォルダーをインポートするか、ウィンドウにドラッグしてください。",
            "아직 아무것도 없어요. 파일이나 폴더를 가져오거나 창으로 끌어다 놓으세요.",
            "Todavía no hay nada: importa archivos o una carpeta entera, o arrástralos a la ventana.",
        ],
    ),
    (
        "Datei fehlt",
        [
            "File missing",
            "ファイルが見つかりません",
            "파일 없음",
            "Falta el archivo",
        ],
    ),
    (
        "In dieses Projekt übernehmen",
        [
            "Add to this project",
            "このプロジェクトに取り込む",
            "이 프로젝트로 가져오기",
            "Añadir a este proyecto",
        ],
    ),
    (
        "Aus der Bibliothek entfernen",
        [
            "Remove from the library",
            "ライブラリから削除",
            "라이브러리에서 제거",
            "Quitar de la biblioteca",
        ],
    ),
    (
        "In die globale Bibliothek kopieren",
        [
            "Copy to the global library",
            "グローバルライブラリにコピー",
            "전역 라이브러리로 복사",
            "Copiar a la biblioteca global",
        ],
    ),
    (
        "Aus dem Projekt entfernen",
        [
            "Remove from the project",
            "プロジェクトから削除",
            "프로젝트에서 제거",
            "Quitar del proyecto",
        ],
    ),
    ("Beispiel", ["Example", "サンプル", "예시", "Ejemplo"]),
    (
        "In der globalen Bibliothek gespeichert",
        [
            "Saved in the global library",
            "グローバルライブラリに保存しました",
            "전역 라이브러리에 저장했어요",
            "Guardado en la biblioteca global",
        ],
    ),
    (
        "Audio hat kein Bild",
        [
            "Audio has no picture",
            "オーディオには映像がありません",
            "오디오에는 화면이 없어요",
            "El audio no tiene imagen",
        ],
    ),
    (
        "HD · 720 Pixel kurze Seite",
        [
            "HD · 720 pixels short side",
            "HD · 短辺720ピクセル",
            "HD · 짧은 변 720픽셀",
            "HD · 720 píxeles lado corto",
        ],
    ),
    (
        "Full HD · 1080",
        [
            "Full HD · 1080",
            "フルHD · 1080",
            "풀 HD · 1080",
            "Full HD · 1080",
        ],
    ),
    (
        "4K · 2160",
        ["4K · 2160", "4K · 2160", "4K · 2160", "4K · 2160"],
    ),
    (
        "Gut (kleine Datei)",
        [
            "Good (small file)",
            "良い（小さいファイル）",
            "좋음 (작은 파일)",
            "Buena (archivo pequeño)",
        ],
    ),
    (
        "Sehr gut",
        ["Very good", "非常に良い", "매우 좋음", "Muy buena"],
    ),
    (
        "Beste Qualität",
        ["Best quality", "最高品質", "최고 품질", "Mejor calidad"],
    ),
    (
        "Wie das Projekt",
        [
            "Same as project",
            "プロジェクトと同じ",
            "프로젝트와 동일",
            "Como el proyecto",
        ],
    ),
    (
        "16:9 Querformat",
        [
            "16:9 landscape",
            "16:9 横向き",
            "16:9 가로",
            "16:9 horizontal",
        ],
    ),
    (
        "9:16 Short · Reels · TikTok",
        [
            "9:16 Short · Reels · TikTok",
            "9:16 ショート · リール · TikTok",
            "9:16 쇼츠 · 릴스 · 틱톡",
            "9:16 Short · Reels · TikTok",
        ],
    ),
    (
        "1:1 Quadrat",
        ["1:1 square", "1:1 正方形", "1:1 정사각형", "1:1 cuadrado"],
    ),
    (
        "4:5 Hochformat",
        ["4:5 portrait", "4:5 縦向き", "4:5 세로", "4:5 vertical"],
    ),
    (
        "23,976 (Kino, NTSC)",
        [
            "23.976 (cinema, NTSC)",
            "23.976 (映画、NTSC)",
            "23.976 (영화, NTSC)",
            "23,976 (cine, NTSC)",
        ],
    ),
    (
        "24 (Kino, Midjourney)",
        [
            "24 (cinema, Midjourney)",
            "24 (映画、Midjourney)",
            "24 (영화, Midjourney)",
            "24 (cine, Midjourney)",
        ],
    ),
    ("25 (PAL)", ["25 (PAL)", "25 (PAL)", "25 (PAL)", "25 (PAL)"]),
    (
        "29,97 (NTSC)",
        [
            "29.97 (NTSC)",
            "29.97 (NTSC)",
            "29.97 (NTSC)",
            "29,97 (NTSC)",
        ],
    ),
    ("30", ["30", "30", "30", "30"]),
    ("50", ["50", "50", "50", "50"]),
    ("59,94", ["59.94", "59.94", "59.94", "59,94"]),
    ("60", ["60", "60", "60", "60"]),
    (
        "48 kHz (Standard für Video)",
        [
            "48 kHz (standard for video)",
            "48 kHz（動画の標準）",
            "48 kHz (영상 표준)",
            "48 kHz (estándar para vídeo)",
        ],
    ),
    (
        "44,1 kHz (Musik, CD)",
        [
            "44.1 kHz (music, CD)",
            "44.1 kHz（音楽、CD）",
            "44.1 kHz (음악, CD)",
            "44,1 kHz (música, CD)",
        ],
    ),
    (
        "32 kHz (klein, Sprache)",
        [
            "32 kHz (small, speech)",
            "32 kHz（小さめ、音声向け）",
            "32 kHz (작음, 음성용)",
            "32 kHz (pequeño, voz)",
        ],
    ),
    (
        "Dateityp wird nicht unterstützt:",
        [
            "File type is not supported:",
            "このファイル形式には対応していません:",
            "지원하지 않는 파일 형식이에요:",
            "Tipo de archivo no compatible:",
        ],
    ),
    (
        "Pfad ist kein gültiger Text",
        [
            "The path is not valid text",
            "パスが有効な文字列ではありません",
            "경로가 올바른 텍스트가 아니에요",
            "La ruta no es un texto válido",
        ],
    ),
    (
        "Ungültige Bildgröße",
        [
            "Invalid picture size",
            "画像サイズが無効です",
            "화면 크기가 올바르지 않아요",
            "Tamaño de imagen no válido",
        ],
    ),
    (
        "Video-Ausgabe nicht möglich",
        [
            "Video output not possible",
            "ビデオ出力ができません",
            "비디오 출력을 할 수 없어요",
            "Salida de vídeo no posible",
        ],
    ),
    (
        "Codec nicht verfügbar",
        [
            "Codec not available",
            "コーデックが利用できません",
            "코덱을 사용할 수 없어요",
            "Códec no disponible",
        ],
    ),
    (
        "Video-Eingang nicht möglich",
        [
            "Video input not possible",
            "ビデオ入力ができません",
            "비디오 입력을 할 수 없어요",
            "Entrada de vídeo no posible",
        ],
    ),
    (
        "Audio-Eingang nicht möglich",
        [
            "Audio input not possible",
            "オーディオ入力ができません",
            "오디오 입력을 할 수 없어요",
            "Entrada de audio no posible",
        ],
    ),
    (
        "Bild zu klein für den Export",
        [
            "Picture too small for export",
            "書き出すには画像が小さすぎます",
            "내보내기에는 화면이 너무 작아요",
            "Imagen demasiado pequeña para exportar",
        ],
    ),
    (
        "Audioformat nicht verfügbar",
        [
            "Audio format not available",
            "オーディオ形式が利用できません",
            "오디오 형식을 사용할 수 없어요",
            "Formato de audio no disponible",
        ],
    ),
    (
        "Überblenden",
        [
            "Crossfade",
            "クロスフェード",
            "크로스페이드",
            "Fundido cruzado",
        ],
    ),
    (
        "Weißblende",
        [
            "Dip to white",
            "ホワイトフェード",
            "화이트 페이드",
            "Fundido a blanco",
        ],
    ),
    (
        "Kreis öffnen",
        [
            "Iris open",
            "円形に開く",
            "원형으로 열기",
            "Apertura circular",
        ],
    ),
    (
        "Flügeltür",
        ["Barn doors", "観音開き", "여닫이문", "Puertas dobles"],
    ),
    (
        "Weiches Überblenden",
        [
            "Soft crossfade",
            "やわらかなクロスフェード",
            "부드러운 크로스페이드",
            "Fundido cruzado suave",
        ],
    ),
    (
        "Pfotenabdrücke tapsen durchs Bild",
        [
            "Paw prints tiptoe across the picture",
            "肉球の足跡がぽてぽてと画面を横切ります",
            "발자국이 총총 화면을 가로질러요",
            "Huellas de patitas recorren la imagen",
        ],
    ),
    (
        "Das Kätzchen flitzt durchs Bild",
        [
            "The kitten dashes across the picture",
            "子猫が画面を駆け抜けます",
            "새끼 고양이가 화면을 쌩 지나가요",
            "El gatito cruza la imagen a toda velocidad",
        ],
    ),
    (
        "Pixelmiau mit Bildstörung",
        [
            "Pixel meow with picture glitch",
            "画面が乱れるピクセルニャー",
            "화면이 깨지는 픽셀 야옹",
            "Maullido de píxeles con interferencias",
        ],
    ),
    (
        "Über Schwarz zum nächsten Bild",
        [
            "Through black to the next picture",
            "黒を経て次の映像へ",
            "검은 화면을 거쳐 다음 화면으로",
            "Pasa por negro a la siguiente imagen",
        ],
    ),
    (
        "Hineinzoomen und überblenden",
        [
            "Zoom in and cross-fade",
            "ズームインしてクロスフェード",
            "확대하며 크로스페이드",
            "Acercar y fundir",
        ],
    ),
    (
        "Ein Kreis öffnet das neue Bild",
        [
            "A circle opens up the new picture",
            "円が広がって新しい映像が現れます",
            "원이 열리며 새 화면이 나타나요",
            "Un círculo abre la nueva imagen",
        ],
    ),
    (
        "Das Bild zerfällt in Pixel",
        [
            "The picture breaks into pixels",
            "映像がピクセルに崩れます",
            "화면이 픽셀로 흩어져요",
            "La imagen se deshace en píxeles",
        ],
    ),
    (
        "Unscharf überblenden",
        [
            "Blurry crossfade",
            "ぼかしてクロスフェード",
            "흐려지며 크로스페이드",
            "Fundido con desenfoque",
        ],
    ),
    (
        "Das Bild öffnet sich wie eine Tür",
        [
            "The picture opens like a door",
            "映像がドアのように開きます",
            "화면이 문처럼 열려요",
            "La imagen se abre como una puerta",
        ],
    ),
    (
        "Der Schnitt liegt außerhalb des Clips",
        [
            "The cut lies outside the clip",
            "分割位置がクリップの外にあります",
            "나누는 위치가 클립 밖에 있어요",
            "El corte queda fuera del clip",
        ],
    ),
    ("Füllen", ["Fill", "塗りつぶし", "채우기", "Rellenar"]),
    (
        "Unscharf füllen",
        [
            "Blurred fill",
            "ぼかして塗りつぶし",
            "흐리게 채우기",
            "Relleno desenfocado",
        ],
    ),
    (
        "Mein Kätzchen-Film",
        [
            "My kitten movie",
            "私の子猫ムービー",
            "나의 고양이 영화",
            "Mi película gatuna",
        ],
    ),
    (
        "Kein gültiges Happy-Kitten-Projekt:",
        [
            "Not a valid Happy-Kitten project:",
            "有効なHappy-Kittenプロジェクトではありません:",
            "올바른 Happy-Kitten 프로젝트가 아니에요:",
            "No es un proyecto válido de Happy-Kitten:",
        ],
    ),
    (
        "Codec nicht unterstützt:",
        [
            "Codec not supported:",
            "コーデックに対応していません:",
            "지원하지 않는 코덱이에요:",
            "Códec no compatible:",
        ],
    ),
    (
        "Die Datei enthält keinen hörbaren Ton",
        [
            "The file contains no audible sound",
            "このファイルには聞こえる音声がありません",
            "이 파일에는 들리는 소리가 없어요",
            "El archivo no contiene sonido audible",
        ],
    ),
    (
        "Claw Wipe",
        ["Claw wipe", "クローワイプ", "발톱 와이프", "Zarpazo"],
    ),
    (
        "Furry Blur",
        [
            "Furry blur",
            "ふわふわブラー",
            "복실복실 블러",
            "Desenfoque peludo",
        ],
    ),
    (
        "Paw Spin",
        ["Paw spin", "ポースピン", "발바닥 스핀", "Giro de patita"],
    ),
    (
        "Gentle Paw Fade",
        [
            "Gentle paw fade",
            "やさしい肉球フェード",
            "포근한 발바닥 페이드",
            "Fundido de patita suave",
        ],
    ),
    (
        "Kitty Dash",
        [
            "Kitty dash",
            "キティダッシュ",
            "키티 대시",
            "Carrera gatuna",
        ],
    ),
    (
        "Meow Glitch",
        [
            "Meow glitch",
            "ニャーグリッチ",
            "야옹 글리치",
            "Glitch maullante",
        ],
    ),
    (
        "Schwarzblende",
        [
            "Dip to black",
            "ブラックフェード",
            "블랙 페이드",
            "Fundido a negro",
        ],
    ),
    (
        "Wischen →",
        ["Wipe →", "ワイプ →", "와이프 →", "Cortinilla →"],
    ),
    (
        "Wischen ↓",
        ["Wipe ↓", "ワイプ ↓", "와이프 ↓", "Cortinilla ↓"],
    ),
    ("Schieben", ["Push", "プッシュ", "밀기", "Empujar"]),
    (
        "Uhrzeiger",
        ["Clock wipe", "時計ワイプ", "시계 방향", "Reloj"],
    ),
    ("Verpixeln", ["Pixelate", "モザイク", "픽셀화", "Pixelar"]),
    ("Verwirbeln", ["Swirl", "渦巻き", "소용돌이", "Remolino"]),
    ("Weichzeichner", ["Blur", "ぼかし", "블러", "Desenfoque"]),
    (
        "Krallen kratzen das Bild auf",
        [
            "Claws scratch the picture open",
            "爪が映像を引っかきます",
            "발톱이 화면을 긁어내요",
            "Las garras rasgan la imagen",
        ],
    ),
    (
        "Flauschiger Fell-Nebel",
        [
            "Fluffy fur mist",
            "ふわふわの毛並みの霧",
            "폭신한 털 안개",
            "Neblina de pelaje esponjoso",
        ],
    ),
    (
        "Riesenpfote wirbelt herein",
        [
            "A giant paw whirls in",
            "巨大な肉球が渦を巻いて現れます",
            "커다란 발바닥이 빙글빙글 들어와요",
            "Una pata gigante entra girando",
        ],
    ),
    (
        "Heller Blitz dazwischen",
        [
            "A bright flash in between",
            "間に明るい閃光が入ります",
            "사이에 밝은 섬광이 들어가요",
            "Un destello brillante en medio",
        ],
    ),
    (
        "Weicher Wisch nach rechts",
        [
            "Soft wipe to the right",
            "右へやわらかくワイプ",
            "오른쪽으로 부드럽게 와이프",
            "Cortinilla suave hacia la derecha",
        ],
    ),
    (
        "Weicher Wisch nach unten",
        [
            "Soft wipe downwards",
            "下へやわらかくワイプ",
            "아래로 부드럽게 와이프",
            "Cortinilla suave hacia abajo",
        ],
    ),
    (
        "Das neue Bild schiebt das alte raus",
        [
            "The new picture pushes the old one out",
            "新しい映像が古い映像を押し出します",
            "새 화면이 이전 화면을 밀어내요",
            "La nueva imagen empuja a la anterior",
        ],
    ),
    (
        "Ein Zeiger wischt rundherum",
        [
            "A hand sweeps all the way round",
            "針がぐるりと一周して切り替えます",
            "시곗바늘이 한 바퀴 돌며 바꿔요",
            "Una aguja barre en círculo",
        ],
    ),
    (
        "Ein Strudel zieht das Bild um",
        [
            "A whirlpool pulls the picture around",
            "渦が映像を巻き込みます",
            "소용돌이가 화면을 휘감아요",
            "Un remolino arrastra la imagen",
        ],
    ),
    ("Ohne", ["None", "なし", "없음", "Ninguno"]),
    ("Weichzeichnen", ["Blur", "ぼかし", "흐림", "Desenfocar"]),
    (
        "Hochgleiten",
        [
            "Slide up",
            "上へスライド",
            "위로 슬라이드",
            "Deslizar hacia arriba",
        ],
    ),
    ("Heranzoomen", ["Zoom in", "ズームイン", "확대", "Acercar"]),
    ("Glitch", ["Glitch", "グリッチ", "글리치", "Glitch"]),
    ("Einpassen", ["Fit", "全体表示", "맞추기", "Ajustar"]),
    (
        "Rund & niedlich",
        [
            "Round & cute",
            "まるくてかわいい",
            "둥글고 귀여운",
            "Redonda y mona",
        ],
    ),
    (
        "Handschrift",
        ["Handwriting", "手書き", "손글씨", "Manuscrita"],
    ),
    ("Kreide", ["Chalk", "チョーク", "분필", "Tiza"]),
    (
        "Arial Fett",
        ["Arial Bold", "Arial 太字", "Arial 굵게", "Arial Negrita"],
    ),
    (
        "Georgia Fett",
        [
            "Georgia Bold",
            "Georgia 太字",
            "Georgia 굵게",
            "Georgia Negrita",
        ],
    ),
    ("verwendet", ["used", "使用済み", "사용됨", "usado"]),
    ("ohne", ["none", "なし", "없음", "ninguno"]),
    ("Sprache", ["Language", "言語", "언어", "Idioma"]),
];
