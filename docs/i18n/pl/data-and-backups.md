# Dane i kopie zapasowe

## Gdzie leżą pliki {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Ustawienia (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Biblioteka, playlisty, carty i sesja (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Pamięć podręczna analizy | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Logi i raporty o awariach | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Ścieżki w Linuksie podążają za zmiennymi XDG (`XDG_CONFIG_HOME` itd.), gdy
są ustawione.

Snap trzyma swoje pliki we własnym folderze: ustawienia w
`~/snap/fauste-player/current/.config/fausteplayer/`, bazę danych i logi w
`~/snap/fauste-player/current/.local/share/fausteplayer/`, a pamięć
podręczną analizy w `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` usuwa je także (na jakiś czas zachowuje
migawkę; `--purge` nie).

## Tryb przenośny {#portable-mode}

Ustaw zmienną środowiskową `FAUSTE_HOME` na folder, a wszystko będzie
przechowywane tam, w `config/`, `data/`, `cache/` i `logs/`. Przydaje się to
na pendrivie albo do trzymania osobnych konfiguracji obok siebie.

## Pliki {#files}

| Plik | Zawartość |
|---|---|
| `config.json` | Ustawienia, wyjścia, progi analizy, zaawansowane strojenie |
| `fauste.db` | Biblioteka (utwory, ich kategorie, markery ręczne, tagi, głośność), playlisty ze znacznikami odtworzenia, strony cartów, rotacje automatyki, ramówkę i dziennik emisji (przechowywany `automation.as_run_keep_days` dni, domyślnie 90) oraz sesja: dla każdego odtwarzacza pokazywana playlista, bieżący i następny utwór, tryb, pozycja, głośność i szerokości kolumn (według kolumny) oraz układ okna |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Kopie bazy danych (zob. niżej) |
| `window.ron` | Rozmiar i położenie głównego okna oraz informacja, czy było zmaksymalizowane (tylko komputer; w Waylandzie położenie wybiera system) |

## Autozapis i kopie zapasowe {#autosave-and-backups}

- Zmiany są zapisywane mniej więcej sekundę po ich wystąpieniu i zapisywane
  jest tylko to, co się zmieniło. Gdy cokolwiek gra, sesja (pozycje) jest
  odświeżana w tym samym tempie.
- Baza danych jest zapisywana w transakcjach, więc utrata zasilania nigdy nie
  zostawia na wpół zapisanej zmiany.
- Przy pierwszym zapisie każdej sesji, a potem mniej więcej co pięć minut,
  gdy zmiany są zapisywane, aplikacja kopiuje bazę do `fauste.db.bak1` i
  przesuwa starsze kopie wyżej, do `.bak2` i `.bak3`. Kilkukrotne
  uruchomienie z rzędu wypycha więc starsze kopie.
- Jeśli bazy danych nie da się odczytać, używana jest najnowsza dobra kopia
  zapasowa. Nieczytelny plik jest zachowywany, przemianowany na
  `fauste.db.corrupt-<time>`, do analizy. Jeśli zawiodą wszystkie kopie,
  aplikacja uruchamia się pusta. Zawsze się uruchamia.
- Kopia, której nie da się użyć, też jest zachowywana: gdy aplikacja
  uruchamia się z innej kopii (lub z pustej bazy), nieużyteczne kopie są
  przemianowywane na `fauste.db.bak1.corrupt-<time>` (lub `.newer-<time>`),
  aby następna kopia zapasowa nigdy ich nie wypchnęła.
- Jeśli inna kopia aplikacji używa bazy danych, ta działa bez zapisywania i
  mówi o tym przy uruchomieniu.
- Baza danych zapisana przez nowszą wersję aplikacji nie jest otwierana:
  zostaje zachowana jako `fauste.db.newer-<time>`, a aplikacja uruchamia się z
  najnowszej kopii, którą potrafi odczytać, albo pusta, gdy żadnej nie ma,
  więc otwarcie danych starszą wersją nigdy ich nie niszczy.
- `config.json` jest zapisywany tak samo jak dawniej: plik tymczasowy
  zastępuje stary, a poprzednie wersje są trzymane obok jako `.bak1`, `.bak2`
  i `.bak3`.
- Jeśli bazy danych nie da się w ogóle użyć w danej sesji (na przykład nie da
  się jej przemianować, bo folder jest tylko do odczytu), aplikacja działa bez
  zapisywania do następnego uruchomienia, a log o tym mówi.

## Playlisty i sesja z wcześniejszej wersji {#playlists-and-session-from-an-earlier-version}

Wcześniejsze wersje trzymały playlisty w `playlists.json`, strony cartów w
`carts.json`, a sesję w `session.json`. Pierwsze uruchomienie tej wersji
odczytuje je (razem z ich kopiami `.bak`) jednorazowo i umieszcza wszystko w
`fauste.db`: bibliotekę z markerami, tagami i głośnością, playlisty ze
znacznikami odtworzenia, strony cartów i sesję. Potem stare pliki są
przemianowywane z `.imported` na końcu (`playlists.json.imported`,
`playlists.json.bak1.imported` i tak dalej). Nie są już używane i możesz je
usunąć, gdy upewnisz się, że wszystko tam jest.

Aby wrócić do wcześniejszej wersji:

1. Zamknij aplikację.
2. Przenieś `fauste.db`, `fauste.db-wal`, `fauste.db-shm` i każdy
   `fauste.db.bak*` do folderu poza folderem danych. Nie usuwaj ich.
3. Usuń `.imported` z nazw starych plików.
4. Uruchom starszą wersję.

Wszystko zapisane od aktualizacji zostaje tylko w folderze, do którego
przeniosłeś pliki: stare pliki tego nie mają. Jeśli później zaktualizujesz
ponownie, stare pliki zostaną zaimportowane jeszcze raz, o ile w folderze
danych nie ma `fauste.db` ani `fauste.db.bak*`.

Jeśli uruchomienie zostanie przerwane podczas importu, następne go dokończy.
Jeśli jednego ze starych plików nie da się odczytać (jest za duży albo został
zapisany przez nowszą wersję), aplikacja nic nie importuje i działa bez
zapisywania tej sesji, więc stare pliki zostają tam, gdzie są; log podaje
nazwę pliku. Napraw go lub usuń i uruchom ponownie. Gdy `fauste.db` już
istnieje, stare pliki JSON nigdy nie są importowane ponownie.

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
