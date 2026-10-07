# Zdalne sterowanie

Fauste Player można odczytywać i obsługiwać przez sieć za pomocą API HTTP,
z aktualizacjami na żywo, oraz przez OSC. Mogą z nich korzystać strona WWW,
aplikacja na telefon, automatyka stacji lub powierzchnia sterująca.
Jest **wyłączone**, dopóki go nie włączysz, a na początku odpowiada tylko na
tym komputerze.

## Włączanie {#turning-it-on}

![Ustawienia, Zdalne: API HTTP włączone i nasłuchujące na tym komputerze oraz OSC wyłączone](../../images/guide/settings-remote.png)

Otwórz **Ustawienia → Zdalne** i zaznacz **Zezwalaj na sterowanie zdalne przez
HTTP** (lub **Zezwalaj na sterowanie przez OSC**). Wiersz pod każdym
przełącznikiem mówi, czy serwer nasłuchuje i gdzie, albo dlaczego nie
wystartował. Zmiany działają od razu; restart nie jest potrzebny. Pole
tekstowe (adres, token, lista) jest stosowane, gdy je opuścisz, otworzysz
inną sekcję lub zamkniesz Ustawienia; wartość, która jeszcze nie jest
prawidłowa, zachowuje tę używaną, a Esc anuluje to, co wpisałeś.

Możesz też edytować `config.json`, gdy Fauste Player jest zamknięty (gdzie
jest, zob. [Dane i kopie zapasowe](data-and-backups.md)). W obiekcie
`"config"` ustaw `remote.http.enabled` na `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Nasłuchuje na `http://127.0.0.1:7380`. Jego uruchomienie nigdy niczego nie
odtwarza; działają tylko żądania.

## Nasłuch w sieci studia {#listening-on-the-studio-network}

Aby dotrzeć do niego z innych komputerów, ustaw `bind` na `0.0.0.0` (albo
jeden z adresów tego komputera) i ustaw **token** o długości co najmniej 16
znaków. W Ustawienia → Zdalne **Generuj** tworzy długi losowy token. Jest
ukryty, dopóki nie naciśniesz **Pokaż**, a **Kopiuj** umieszcza go w
schowku dla klienta. Bez tokenu serwer odmawia startu, a log mówi dlaczego.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Klienci wysyłają token jako `Authorization: Bearer <token>`. API nie jest
szyfrowane. Trzymaj je w zaufanej sieci studia albo umieść za zwrotnym
proxy z HTTPS.

## Strony WWW {#web-pages}

Strona WWW serwowana z innego adresu może używać API tylko wtedy, gdy jej
origin (na przykład `https://studio.example`) jest wymieniony w
`cors_origins`. Żądania z innych stron są odrzucane, nawet na tym komputerze,
więc strona, którą akurat masz otwartą, nie może sterować odtwarzaczem. `"*"`
(dowolny origin) jest akceptowane tylko razem z tokenem.

## Co może zrobić klient {#what-a-client-can-do}

Klient może:

- odczytywać odtwarzacze, playlisty, utwory (z okładką i przebiegiem) oraz
  cartwall;
- odtwarzać, wstrzymywać, zatrzymywać, wyciszać, uruchamiać od początku i
  cofać się;
- wybrać następną pozycję (także tę na antenie: zagra jeszcze raz), odsłuchiwać wstępnie i przewijać (w zatrzymanym odtwarzaczu przewinięcie wybiera, skąd Play uruchomi następną pozycję, a przewinięcie przed jej cue-in zaczyna w cue-in);
- ustawiać głośności, tryby, stop po bieżącym oraz oznaczenia powtarzania i
  stopu po pozycji;
- odpalać, zatrzymywać i odsłuchiwać wstępnie carty oraz zmieniać pokazywaną
  stronę cartów;
- edytować: tworzyć, zmieniać nazwy i usuwać playlisty; dodawać utwór, który
  jest już wczytany, oraz usuwać, przenosić lub duplikować pozycje; tworzyć,
  zmieniać nazwy, zmieniać rozmiar i usuwać strony cartów oraz konfigurować
  cart wczytanym utworem; ustawiać lub resetować markery.

Usunięcie tego, co jest na antenie, jest odrzucane, tak jak na ekranie.
Plików, które nie są jeszcze wczytane, nie można dodawać zdalnie: leżą na tym
komputerze, więc najpierw dodaj je tutaj.

Przycisk wyszarzony na ekranie jest odrzucany także zdalnie. Pełna
dokumentacja jest w [dokumentacji technicznej](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Wypróbuj z terminala:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Aktualizacje na żywo {#live-updates}

Klient może śledzić zmiany na bieżąco, zamiast pytać w kółko.
`GET /api/v1/events` to strumień zdarzeń: najpierw cały stan, potem każda
zmiana odtwarzacza, playlisty, utworu lub cartwalla oraz czasy tego, co gra,
kilka razy na sekundę.

    curl -sN http://127.0.0.1:7380/api/v1/events

Strona WWW używa `EventSource`. Przeglądarki nie mogą tam wysłać tokenu jako
nagłówka, więc idzie w adresie: `/api/v1/events?token=<token>`.

## OSC {#osc}

OSC to zwykły protokół powierzchni sterujących, pulpitów oświetleniowych i
oprogramowania do sterowania pokazami. Włącz go przez `remote.osc.enabled`.
Nasłuchuje na porcie UDP 7381 tego komputera. Aby przyjmować pakiety z innych
komputerów, ustaw `remote.osc.bind` na `0.0.0.0` i wypisz ich adresy lub
podsieci w `remote.osc.allowed_sources` (na przykład `"192.168.1.0/24"`). OSC
nie ma hasła, więc trzymaj go w zaufanej sieci studia.

Odtwarzacze są numerowane 1, 2, 3… tak, jak pojawiają się na ekranie. Carty
są numerowane na pokazywanej stronie.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Powierzchnia, która chce pokazywać stan (diody, nazwy, odliczania),
subskrybuje, a potem otrzymuje każdą wartość raz, a następnie tylko to, co
się zmienia. Gdy odtwarzacz lub przycisk carta znika (mniej odtwarzaczy,
mniejsza strona), jego adresy otrzymują raz pustą wartość, aby powierzchnia
je wyczyściła. Musi subskrybować ponownie w ciągu minuty
(`subscription_ttl_secs`), aby dalej otrzymywać:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Subskrybent może wskazać dowolny port własnego adresu, a zachowywanych jest
do `max_subscribers`. Każdy, kto może wysyłać, może więc też subskrybować. To
kolejny powód, by trzymać OSC w zaufanej sieci.

`oscsend` i `oscdump` pochodzą z liblo (`liblo-tools` w Debianie i Ubuntu).
Pełna lista adresów jest w
[dokumentacji technicznej](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

## Wszystkie ustawienia {#all-settings}

| Ustawienie | Domyślnie | Znaczenie |
|---|---|---|
| `remote.http.enabled` | `false` | Włącza API |
| `remote.http.bind` | `127.0.0.1` | Adres nasłuchu (adres IP) |
| `remote.http.port` | `7380` | Port (1024–65535) |
| `remote.http.token` | puste | Wymagany poza tym komputerem; co najmniej 16 znaków |
| `remote.http.cors_origins` | brak | Originy WWW, które mogą wywoływać API |
| `remote.http.request_timeout_ms` | `10000` | Najdłuższy czas, jaki może zająć żądanie |
| `remote.http.max_body_bytes` | `65536` | Największe ciało żądania |
| `remote.http.max_event_clients` | `16` | Strumienie zdarzeń na żywo naraz |
| `remote.osc.enabled` | `false` | Włącza OSC |
| `remote.osc.bind` | `127.0.0.1` | Adres nasłuchu |
| `remote.osc.port` | `7381` | Port UDP (1024–65535) |
| `remote.osc.allowed_sources` | ten komputer | Adresy lub podsieci, których pakiety są przyjmowane |
| `remote.osc.max_subscribers` | `16` | Subskrybenci naraz |
| `remote.osc.subscription_ttl_secs` | `60` | Subskrypcja nieodnowiona w tym czasie wygasa |
| `remote.events.position_interval_ms` | `250` | Jak często czasy są publikowane podczas odtwarzania |

Wartości spoza zakresu są poprawiane przy wczytywaniu pliku, a poprawka jest
zapisywana w logu.
