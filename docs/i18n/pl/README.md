# Fauste Player — Podręcznik użytkownika

<div class="ai-notice" role="note">Ten podręcznik został przetłumaczony za pomocą AI z angielskiego podręcznika (v0.14.0-144-g8517d951, commit 8517d95) i może zawierać błędy. W razie różnic obowiązuje wersja angielska. <a href="../index.html" hreflang="en">Przeczytaj oryginał po angielsku</a></div>

Fauste Player to aplikacja do emisji (playout) dla studiów radiowych.
Uruchamia kilka niezależnych **odtwarzaczy** obok siebie. Każdy odtwarzacz
ma własne playlisty i własny transport. Odtwarzacze potrafią łączyć utwory
miksami z nakładaniem, odsłuchiwać wstępnie na osobnym wyjściu **CUE** i
przetrwać awarie, nie wypuszczając niczego na antenę samodzielnie.

> **Stworzone z pomocą AI, udostępniane w stanie, w jakim jest.** Fauste
> Player, ten podręcznik i jego tłumaczenia powstały przy znacznej pomocy
> narzędzi AI. Program nie ma żadnej gwarancji, a używasz go na własne
> ryzyko: zobacz [Zastrzeżenie](https://github.com/sanchezfauste/fauste-player#disclaimer).

![Ekran główny](../../images/main-screen.png)

| Podręcznik | Czego dotyczy |
|---|---|
| [Pierwsze kroki](getting-started.md) | Instalacja, pierwsze uruchomienie, dodawanie muzyki, odtwarzanie |
| [Odtwarzacze](players.md) | Kolumna odtwarzacza: transport, tryby, odliczanie, przebieg, CUE |
| [Playlisty](playlists.md) | Karty, tabela utworów, przeciąganie i upuszczanie, menu kontekstowe, pliki M3U/PLS |
| [Cartwall](cartwall.md) | Carty natychmiastowe: odpalanie, strony, pętle, carty wyłączne, konfiguracja |
| [Biblioteka](library.md) | Okno biblioteki utworów: kategorie, rodzaje, wyszukiwanie, odsłuch wstępny, zewnętrzny edytor i zmienione pliki |
| [Układ](layout.md) | Pokazywanie i ukrywanie paneli, szerokości odtwarzaczy, przenoszenie i zmiana rozmiaru cartwalla, maksymalizowanie panelu, menu układu |
| [Automatyzacja](automation.md) | Kolumna Ramówka, jej edytor, wypełniacz muzyczny i jego rotacje |
| [Markery i miksowanie](markers-and-mixing.md) | Cue-in/cue-out, punkt MIX, intro i outro, automatyczna analiza |
| [Ustawienia](settings.md) | Wyjścia audio, odtwarzacze, analiza, playlisty, MIDI, zdalne sterowanie |
| [Normalizacja głośności](loudness.md) | Każdy utwór i cart z tą samą głośnością przy stałym wzmocnieniu dla każdego utworu: znaczek wzmocnienia, wyłączanie i korekta utworu, wyjścia bit-perfect |
| [Wyjście bit-perfect](bit-perfect.md) | Specjalny tryb: bez przenikania, głośność na 100 %. Urządzenia na wyłączność dostosowujące się do częstotliwości każdego pliku, wskaźnik BP, wyjście DSD, sprawdzanie toru |
| [Klawiatura](keyboard.md) | Skróty |
| [Powierzchnie sterujące MIDI](midi.md) | Przyciski, suwaki i diody w kontrolerach MIDI |
| [Zdalne sterowanie](remote-control.md) | Obsługa Fauste Player przez sieć: API HTTP, bezpieczeństwo |
| [Dane i kopie zapasowe](data-and-backups.md) | Gdzie leżą pliki, autozapis, odzyskiwanie po awarii, tryb przenośny |
| [Android](android.md) | Aplikacja na telefonach i tabletach: instalacja, układ dotykowy, muzyka na urządzeniu |
| [Rozwiązywanie problemów](troubleshooting.md) | Brak dźwięku, utracone urządzenie, pliki oznaczone jako niedostępne, logi |

Interfejs jest dostępny po angielsku, hiszpańsku, katalońsku, niderlandzku,
francusku, galicyjsku, niemiecku, włosku, baskijsku, polsku i portugalsku.
Podąża za językiem systemu operacyjnego, chyba że wybierzesz inny w
Ustawieniach. Interfejs w każdym języku, także angielskim i hiszpańskim, został
wykonany za pomocą AI i może zawierać błędy. Poprawki są mile widziane jako
zgłoszenia (issues) lub pull requesty.

Podręcznik jest też dostępny w innych językach, przetłumaczony za pomocą AI z
tego angielskiego podręcznika: w [podręczniku online](https://sanchezfauste.com/fauste-player/guide/)
wybierz język przyciskiem z globusem na górnym pasku.
Przetłumaczona strona informuje o tym u góry i odsyła do strony
angielskiej, która jest wiążąca, gdy obie wersje się różnią.
