# Powierzchnie sterujące MIDI

Fauste Player można obsługiwać z kontrolerów MIDI: kontrolerów z padami i
suwakami, powierzchni w stylu DJ-skim lub klawiatur. Przyciski transportu i
suwak głośności każdego odtwarzacza można przypisać do przycisku, klawisza
lub suwaka, a przyciski z diodami pokazują, co robi każdy odtwarzacz.

## Włączanie {#turning-it-on}

![Ustawienia, MIDI: przełącznik włączający MIDI oraz lista akcji, każda z przyciskiem Ucz](../../images/guide/settings-midi.png)

Otwórz **Ustawienia → MIDI** i zaznacz **Używaj kontrolerów MIDI**. Lista pod
**Porty wejściowe** pokazuje każde wejście MIDI, jakie ma komputer, i czy
jest podłączone. Otwierane są tylko kontrolery, które przypisałeś (niektóre
systemy oddają port jednemu programowi naraz), oraz każde wejście, gdy uczysz
kontrolkę; program nigdy nie nasłuchuje własnych portów. Kontrolery można
podłączać i odłączać podczas pracy programu: co kilka sekund porty są
szukane od nowa, a kontroler, który wraca, jest łączony według nazwy, a jego
diody są ustawiane na nowo.

W Linuksie MIDI idzie przez ALSA: Twój użytkownik musi mieć prawo otwierania
sekwencera (`/dev/snd/seq`, zwykle przez członkostwo w grupie `audio`).

## Przypisywanie kontrolki {#binding-a-control}

Dla każdego odtwarzacza jest wiersz na akcję: **Play / Następny**, **Pauza**,
**Stop**, **Stop z wyciszeniem**, **Od początku**, **Poprzedni**, **CUE** i
**Głośność**.

1. Kliknij **Ucz** w wierszu.
2. Naciśnij przycisk lub poruszaj suwakiem, który chcesz (w międzyczasie
   widać **Porusz kontrolką…**). Przycisk przyjmuje klawisz, pad albo
   przycisk wysyłający control change; **Głośność** przyjmuje suwak lub
   pokrętło (control change) albo suwak pitch bend.
3. Wiersz pokazuje urządzenie i kontrolkę, na przykład
   `APC mini · Nuta 36, kanał 1`.

Jeśli ta kontrolka była już przypisana do innej akcji, przechodzi do tej.
`Esc` lub ponowne kliknięcie przycisku anuluje uczenie. **Wyczyść** usuwa
przypisanie.

## Jak zachowują się kontrolki {#how-the-controls-behave}

- Przycisk działa, gdy jest naciśnięty (klawisz lub pad w dół albo control
  change przechodzący w górę przez środek swojego zakresu), dokładnie jak
  przycisk odtwarzacza na ekranie. Przycisk przyciemniony na ekranie nic nie
  robi.
- Suwak zmienia głośność w tej samej skali co suwak na ekranie. Aby uniknąć
  skoków, przejmuje kontrolę dopiero wtedy, gdy dotrze do bieżącej
  głośności lub ją minie (soft takeover); jeśli głośność zmieni się na
  ekranie, suwak musi ją ponownie dogonić. Nic się nie zmienia, dopóki nie
  poruszysz kontrolki: uruchomienie programu nigdy niczego nie wysyła na
  antenę.
- Przy włączonym **Podświetlaj przyciski (LED)** przypisane przyciski się
  świecą: Play, gdy odtwarzacz jest na antenie, Pauza miga podczas
  wstrzymania, CUE podczas odsłuchu wstępnego, a Stop, Stop z wyciszeniem,
  Od początku i Poprzedni, gdy mogą działać. Diody dostają sygnał na porcie
  wyjściowym kontrolera o tej samej nazwie; inny można ustawić w
  `config.json` (`midi.devices`).
