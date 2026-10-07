# Dane i kopie zapasowe

## Gdzie leżą pliki {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Ustawienia (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Playlisty i sesja | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Pamięć podręczna analizy | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Logi i raporty o awariach | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Ścieżki w Linuksie podążają za zmiennymi XDG (`XDG_CONFIG_HOME` itd.), gdy
są ustawione.

## Tryb przenośny {#portable-mode}

Ustaw zmienną środowiskową `FAUSTE_HOME` na folder, a wszystko będzie
przechowywane tam, w `config/`, `data/`, `cache/` i `logs/`. Przydaje się to
na pendrivie albo do trzymania osobnych konfiguracji obok siebie.

## Pliki {#files}

| Plik | Zawartość |
|---|---|
| `config.json` | Ustawienia, wyjścia, progi analizy, zaawansowane strojenie |
| `playlists.json` | Playlisty, utwory, znaczniki odtworzenia, markery ręczne |
| `session.json` | Dla każdego odtwarzacza: pokazywana playlista, bieżący i następny utwór, tryb, pozycja, głośność, szerokości kolumn (według kolumny) |

## Autozapis i kopie zapasowe {#autosave-and-backups}

- Zmiany są zapisywane mniej więcej sekundę po ich wystąpieniu. Gdy cokolwiek
  gra, sesja (pozycje) jest odświeżana w tym samym tempie.
- Każdy zapis tworzy plik tymczasowy, a następnie zastępuje nim stary, więc
  utrata zasilania nigdy nie zostawia na wpół zapisanego pliku.
- Poprzednie wersje są trzymane jako `.bak1`, `.bak2` i `.bak3`.
- Jeśli pliku nie da się odczytać, używana jest najnowsza dobra kopia
  zapasowa. Nieczytelny plik jest zachowywany, przemianowany na
  `*.corrupt-<time>`, do analizy. Aplikacja zawsze się uruchamia.

## Odzyskiwanie po awarii {#crash-recovery}

Po awarii lub ponownym uruchomieniu każdy odtwarzacz wraca ze swoją
playlistą, bieżącym i następnym utworem oraz pozycją, ale **wstrzymany lub
zatrzymany**. Nic nie trafia na antenę samo. Odtwarzacz, który był na samym
końcu utworu, wraca na początek tego utworu, więc naciśnięcie Play go
odtwarza, zamiast od razu go kończyć.

## Ręczna edycja `config.json` {#editing-configjson-by-hand}

Najpierw zamknij aplikację (jeśli dźwięk jest na antenie, zapyta przed
zamknięciem). Nieznane lub spoza zakresu wartości są poprawiane do
najbliższej prawidłowej przy wczytywaniu pliku, a poprawki są zapisywane w
logu. Sekcje `limits` i `tuning` zawierają wartości zaawansowane (limity
zasobów, czasy silnika), których nie ma w oknie Ustawień.
