# Wyjście bit-perfect

Urządzenie **bit-perfect** otrzymuje próbki każdego pliku dokładnie takie,
jakie są w pliku: ta sama częstotliwość próbkowania, te same wartości, bez
resamplingu, zmiany głośności ani miksowania. Przydaje się w torach
odsłuchowych i łączach cyfrowych, gdzie należy unikać jakiejkolwiek obróbki
na komputerze.

## Ustawienie urządzenia jako bit-perfect {#setting-a-device-bit-perfect}

1. W **Ustawienia → Wyjścia audio** wybierz urządzenie jawnie jako wyjście
   Main odtwarzacza (lub cartwalla). Odtwarzacza pozostawionego na
   domyślnym urządzeniu systemowym nie można ustawić jako bit-perfect.
   Urządzenie, którego żadne wyjście już nie używa, traci przełącznik
   bit-perfect i tryb DSD przy następnym uruchomieniu aplikacji.
2. Wybierz **Zaawansowane** u góry sekcji. W części **Ustawienia urządzeń**
   włącz **Bit-perfect** obok urządzenia.
3. Uruchom aplikację ponownie.

Urządzenie startuje wtedy z własną częstotliwością próbkowania, jeśli
nadałeś ją w tym samym miejscu, a w przeciwnym razie z globalną, i stamtąd
podąża za każdym plikiem.

Przełącznik jest wyłączony, gdy urządzenie nie może dać dostępu na
wyłączność.
- **Linux:** wybierz urządzenie ALSA, którego nazwa zaczyna się od `hw:`. To
  sama karta dźwiękowa. PulseAudio, PipeWire, JACK oraz urządzenia ALSA
  `default` lub `plughw:` miksują albo konwertują, więc nigdy nie są
  bit-perfect.
- **Windows:** wybierz urządzenie w systemie **WASAPI**. Jest otwierane w
  trybie wyłącznym.
  - W ustawieniach dźwięku Windows we właściwościach urządzenia, na karcie
    **Zaawansowane**, musi być włączone *Zezwalaj aplikacjom na przejęcie
    wyłącznej kontroli nad tym urządzeniem* (domyślnie jest włączone).
  - Gdy gra, żaden inny program nie może używać urządzenia.
- **macOS:** wybierz urządzenie w **Core Audio**. Jest otwierane w trybie
  hog.
  - Częstotliwość próbkowania urządzenia jest ustawiana na częstotliwość
    utworu, a jego format na najszerszy format całkowitoliczbowy, jaki
    oferuje przy tej częstotliwości (ustawienia, które pokazuje Konfiguracja
    Audio MIDI).
  - Są oddawane, gdy aplikacja przestaje używać urządzenia.
  - Dwóch urządzeń o dokładnie tej samej nazwie nie można ustawić jako
    bit-perfect.

## Co dzieje się na urządzeniu bit-perfect {#what-happens-on-a-bit-perfect-device}

- **Dostęp na wyłączność.** Nic innego na komputerze nie może grać na
  urządzeniu, dopóki aplikacja go używa. Jeśli dostęp na wyłączność zostanie
  odrzucony, urządzenie nadal gra, współdzielone, a wskaźnik BP pozostaje
  wyłączony.
- **Częstotliwość podąża za plikiem.** Gdy na urządzeniu nic nie gra, a
  startuje utwór o innej częstotliwości próbkowania, urządzenie jest
  otwierane ponownie z tą częstotliwością.
  - Dzieje się tak, gdy odtwarzasz utwór, wznawiasz wczytany we wstrzymaniu,
    odsłuchujesz wstępnie albo odpalasz cart. Utwory, które tylko czekają
    (następny utwór każdego odtwarzacza), są przygotowywane od nowa z nową
    częstotliwością.
  - Ponowne otwarcie trwa tyle, ile urządzenie potrzebuje na start
    (zwykle kilkadziesiąt milisekund). O tyle później następuje start.
  - Gdy na urządzeniu coś gra, częstotliwość nigdy się nie zmienia. Utwór o
    innej częstotliwości, który startuje wtedy (na przykład utwór 48 kHz
    miksowany po utworze 44,1 kHz albo utwór uruchomiony, gdy inny
    odtwarzacz lub cart gra na tym samym urządzeniu), jest konwertowany na
    całej długości i nie jest bit-perfect.
  - Jeśli urządzenie odrzuci częstotliwość, zachowuje poprzednią, a utwór
    jest konwertowany.
- **Bez obróbki, gdy nic jej nie wymaga.** Próbki przechodzą bez zmian, dopóki
  spełnione są wszystkie poniższe warunki:
  - głośność odtwarzacza wynosi 100 %;
  - nie trwa żadne wyciszanie;
  - nic innego nie gra na tych samych wyjściach (inny odtwarzacz, cart, ton
    testowy).

## DSD {#dsd}

Plik DSD zwykle gra po konwersji do PCM, jak każdy inny plik. Urządzenie
bit-perfect może natomiast otrzymać strumień DSD bez zmian.

**Trzy tryby.** W widoku Zaawansowane w Ustawienia → Wyjścia audio każde
urządzenie używane przez wyjście ma wybór **DSD** pod przełącznikiem
bit-perfect:
- **Konwertuj na PCM** (domyślnie): DSD jest konwertowane, jak na każdym
  innym urządzeniu.
- **DoP** (DSD over PCM): bity DSD podróżują wewnątrz 24-bitowych próbek PCM,
  które rozpoznaje większość przetworników obsługujących DSD. Działa w każdym
  systemie.
- **Natywne DSD** (tylko Linux): surowe DSD, dla urządzeń ALSA `hw:`, których
  sterownik zgłasza format próbek DSD.

Oferowane są tylko tryby, które urządzenie może przyjąć, a wiersz pod
wyborem mówi, dlaczego inne nie: urządzenie nie jest podłączone, bit-perfect
jest wyłączony, urządzenia nie można otworzyć na wyłączność, natywne DSD
wymaga Linuksa albo urządzenie nie przyjmuje natywnego DSD. Tryb zapisany dla
urządzenia, które nie może go teraz przyjąć, jest pokazywany jako PCM, czyli
to, co faktycznie gra; zapisany tryb wraca, gdy urządzenie znów go przyjmie.
Zmiana trybu, ustawienia miksowania lub ciszy DSD wymaga restartu, tak jak
pozostałe ustawienia wyjść.

**Kiedy DSD wychodzi bez zmian.** Wszystkie poniższe warunki muszą być
spełnione w chwili startu utworu:
- urządzenie jest bit-perfect, z dostępem na wyłączność, a jego tryb to DoP
  lub natywne DSD;
- utwór jest w DSD (DSF lub DFF), mono lub stereo, i został przeanalizowany
  (tak poznaje się jego częstotliwość DSD);
- głośność odtwarzacza wynosi 100 %;
- nic innego nie gra na urządzeniu (inny odtwarzacz, cart, ton testowy);
- urządzenie przyjmuje strumień. DoP wymaga częstotliwości urządzenia równej
  częstotliwości DSD podzielonej przez 16 (176,4 kHz dla DSD64, 352,8 kHz dla
  DSD128, 705,6 kHz dla DSD256) oraz formatu 24- lub 32-bitowego. Natywne DSD
  wymaga urządzenia, które przyjmuje format DSD przy tej częstotliwości.

Gdy natywne DSD się kończy, urządzenie wraca do PCM z częstotliwością, jaką
miało przed utworem DSD, ponieważ wiele przetworników przyjmuje natywne DSD
przy częstotliwościach, których nie potrafi odtwarzać jako PCM (żaden
przetwornik nie gra PCM z częstotliwością, z jaką pracuje DSD512). Utwór,
który toczy się dalej jako PCM, zachowuje częstotliwość strumienia DSD, gdy
urządzenie przyjmuje ją jako PCM, a w przeciwnym razie kontynuuje z
wcześniejszą częstotliwością od miejsca, w którym był, jak wszystko inne
grające na tym urządzeniu; trwające tam wyciszanie kończy się od razu.
Urządzenie nigdy nie zostaje na częstotliwości, której odmawia: jeśli żadna
częstotliwość się nie otworzy (na przykład urządzenie zostało wtedy
odłączone), wyjście jest utracone, aż automatyczna ponowna próba otworzy je
z wcześniejszą częstotliwością.

W przeciwnym razie utwór jest konwertowany do PCM, a log podaje dlaczego (na
przykład „something else plays on the device” albo „the device refused
705600 Hz”). Odsłuch wstępny i carty są zawsze konwertowane.

Gdy DSD wychodzi bez zmian:
- plakietka w nagłówku pokazuje **DSD** zamiast **BP**;
- mierniki pokazują poziom konwersji PCM tego samego utworu, więc działają
  jak zwykle;
- głośność musi pozostać na 100 %: mówi o tym podpowiedź suwaka. Jego
  poruszenie przełącza utwór na PCM (zob. niżej);
- Stop i Stop z wyciszeniem zatrzymują utwór od razu, bez wyciszania, ponieważ
  strumienia DSD nie da się wyciszyć. Naciśnięcie Play na innym utworze, gdy
  ten gra, ucina go w ten sam sposób zamiast przenikać;
- pauza i wznowienie również działają od razu, bez rampy.

**Cisza na krawędziach.** Każdy start, koniec i przejście na PCM najpierw
wysyła ciszę DSD (domyślnie 200 ms), aby przetwornik zsynchronizował się bez
trzasku. Wyjątkiem jest utwór DSD, który kontynuuje strumień tego samego
rodzaju i częstotliwości DSD, którego cisza wciąż trwa: przetwornik jest
nadal zsynchronizowany, więc startuje bez dodatkowej ciszy. Utwór startuje
więc o tyle później, a przejście na PCM zostawia przerwę tej długości. To
**Cisza DSD** w Ustawienia → Wyjścia audio, Zaawansowane (od 0 do 2000 ms).

**Gdy inne źródło potrzebuje urządzenia.** **Gdy inne źródło potrzebuje
wyjścia DSD** w Ustawienia → Wyjścia audio wybiera, co się dzieje, gdy inny
odtwarzacz, cart lub ton testowy startuje na tym samym urządzeniu (wyjątkiem
jest poruszenie własnego suwaka odtwarzacza: zawsze przełącza utwór na PCM):
- **Kontynuuj utwór DSD jako PCM** (domyślnie). Strumień przełącza się na PCM
  po ciszy DSD, a utwór toczy się dalej, skonwertowany, od miejsca, w którym
  był. To samo dzieje się z utworem, który następuje samoczynnie (zob.
  niżej).
- **Zachowaj DSD i wycisz inne źródła.** Nic nie przerywa strumienia DSD.
  Inne źródła skierowane na urządzenie są wyciszone do końca utworu DSD, a
  odtwarzacz w tym czasie pokazuje plakietkę **Inne wyciszone**. Własny
  następny utwór odtwarzacza się nie nakłada: startuje, gdy kończy się utwór
  DSD, bez przenikania i bez segue. Utwór PCM czeka na ciszę DSD; utwór DSD
  tego samego rodzaju i częstotliwości DSD kontynuuje strumień bez niej.
  Poruszenie suwaka nadal przełącza utwór na PCM.

**Album nie pozostaje w DSD przy ustawieniu domyślnym.** Przy *Kontynuuj
utwór DSD jako PCM* jako DSD wychodzi tylko utwór DSD, który startuje na
bezczynnym urządzeniu. Utwory, które odtwarzacz uruchamia sam później (na
końcu utworu, przy segue lub przenikaniu), startują z wstępnego
załadowania, które jest zawsze PCM, więc urządzenie przełącza się na PCM, a
one grają skonwertowane. Utwór, który uruchamiasz sam (Play, dwuklik),
wychodzi znów jako DSD, gdy urządzenie jest bezczynne albo poprzedni
strumień DSD wciąż jest w swojej ciszy przy tej samej częstotliwości DSD.
Aby cały album DSD pozostał DSD, wybierz *Zachowaj DSD i wycisz inne
źródła*. Wtedy każdy utwór odtwarzacza wychodzi jako DSD, a następny startuje,
gdy poprzedni się kończy.

Jeśli urządzenie zostanie utracone podczas gry DSD i wróci niezdolne do jego
przenoszenia (na przykład bez dostępu na wyłączność), utwór toczy się dalej
jako PCM.

## Wskaźnik BP {#the-bp-badge}

Plakietka **BP** w nagłówku odtwarzacza świeci, gdy bieżący utwór dociera do
swojego urządzenia Main bez zmian. Wszystkie poniższe warunki muszą być
spełnione:

- urządzenie jest bit-perfect i otwarte z dostępem na wyłączność;
- urządzenie pracuje z częstotliwością próbkowania utworu;
- utwór to bezstratne całkowitoliczbowe PCM (WAV, AIFF, FLAC, ALAC, WavPack
  lub Monkey's Audio), mono lub stereo, co najwyżej 24-bitowe, a format
  urządzenia mieści jego rozmiar próbki (plik 24-bitowy na urządzeniu
  16-bitowym nie jest bit-perfect). DSD jest konwertowane, więc nigdy nie
  zapala BP; gdy wychodzi bez zmian (zob. [DSD](#dsd)), plakietka pokazuje
  zamiast tego **DSD**;
- utwór został przeanalizowany, ponieważ tak poznaje się jego częstotliwość i
  rozmiar próbki. Utwory przeanalizowane przez wcześniejszą wersję dostają
  swój format po ponownej analizie (komunikat przy starcie lub Ustawienia →
  Analiza) albo gdy tylko pokaże je odtwarzacz lub trzyma je cart;
- głośność wynosi 100 %, nie trwa wyciszanie i nic innego nie gra na tych
  samych wyjściach.

Niektóre pliki nigdy nie są pokazywane jako bit-perfect:
- **Pliki stratne** (MP3, AAC, Ogg Vorbis, Opus): ich zdekodowane próbki nie
  są wartościami całkowitymi, które przyjmuje urządzenie.
- **Pliki powyżej 24 bitów:** mikser pracuje w 32-bitowej liczbie
  zmiennoprzecinkowej, która przenosi 24 bity dokładnie.
- **Pliki z więcej niż dwoma kanałami:** są miksowane do stereo.

## Sprawdzenie samodzielnie {#checking-it-yourself}

Aby zweryfikować tor od końca do końca:

1. Połącz wyjście cyfrowe urządzenia (S/PDIF, AES lub pętlę zwrotną USB) z
   rejestratorem, który nagrywa bit-dokładnie.
2. Odtwórz bezstratny plik testowy przy 100 %, gdy nic innego nie gra.
3. Nagraj go.
4. Porównaj nagranie z plikiem. Na przykład w SoX odwróć jedno i zmiksuj je:
   `sox -m -v 1 file.wav -v -1 recording.wav diff.wav` po wyrównaniu ich
   początków. Każda próbka różnicy musi wynosić zero.

Zautomatyzowane testy projektu sprawdzają tę samą własność wewnątrz
aplikacji, na symulowanym urządzeniu.

### DSD na prawdziwym przetworniku {#dsd-on-a-real-converter}

Zautomatyzowane testy sprawdzają DSD tylko na symulowanych urządzeniach. DoP
i natywne DSD nie były przez projekt wypróbowane na prawdziwym przetworniku.
Aby sprawdzić jeden:
1. Ustaw urządzenie na **DoP** (lub **Natywne DSD** w Linuksie), uruchom
   ponownie i odtwórz plik DSD przy 100 %, gdy nic innego nie gra.
   Nagłówek musi pokazywać **DSD**, a wyświetlacz samego przetwornika
   powinien pokazywać częstotliwość DSD (na przykład DSD64) zamiast
   częstotliwości PCM. Przetwornik, który pokazuje częstotliwość PCM albo
   gra szum, nie rozpoznaje strumienia: wróć do **Konwertuj na PCM**.
2. Posłuchaj, czy nie ma trzasku lub wybuchu szumu na początku, przy Stop,
   na końcu utworu i przy poruszaniu suwakiem. Trzask oznacza, że
   przetwornik potrzebuje dłuższej **Ciszy DSD** (Ustawienia → Wyjścia audio,
   Zaawansowane).
3. Uruchom cart lub inny odtwarzacz na tym samym urządzeniu, raz przy każdym
   ustawieniu miksowania, i sprawdź zachowanie opisane powyżej.
4. W Linuksie, aby sprawdzić natywne DSD bez aplikacji, uruchom
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   Otwiera urządzenie w natywnym DSD przy DSD64 i odtwarza sekundę ciszy
   DSD. Test musi przejść, a przetwornik powinien zsynchronizować się na
   DSD64.
5. Mając pliki DSD w `test-music/`, `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   odtwarza je przez silnik na symulowanym urządzeniu i porównuje słowa z
   bajtami pliku (zob. [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
