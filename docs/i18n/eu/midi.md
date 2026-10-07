# MIDI kontrol-gainazalak

Fauste Player MIDI kontrolagailuetatik erabil daiteke: pad eta fader
kontrolagailuak, DJ estiloko gainazalak edo teklatuak. Erreproduzitzaile
bakoitzaren garraio-botoiak eta bolumen-faderra botoi, tekla edo fader bati
lotu daitezke, eta argidun botoiek erreproduzitzaile bakoitza zertan ari
den erakusten dute.

## Aktibatzea {#turning-it-on}

![Ezarpenak, MIDI: MIDI aktibatzeko etengailua, eta ekintzen zerrenda, bakoitza bere Ikasi botoiarekin](../../images/guide/settings-midi.png)

Ireki **Ezarpenak → MIDI** eta markatu **Erabili MIDI kontrol-gainazalak**.
**Sarrera-atakak** azpiko zerrendak ordenagailuak dituen MIDI sarrera
guztiak erakusten ditu, eta konektatuta dauden ala ez. Lotu dituzun
kontrolagailuak bakarrik irekitzen dira (sistema batzuek ataka bat programa
bakar bati ematen diote aldi berean), eta sarrera guztiak kontrol bat
ikasten ari zaren bitartean; programak ez ditu inoiz bere atakak entzuten.
Kontrolagailuak konektatu edo deskonektatu daitezke programa abian dagoen
bitartean: segundo pare batetik behin atakak berriro bilatzen dira, eta
itzultzen den kontrolagailu bat bere izenaren bidez konektatzen da, bere
argiak berriro ezarrita.

Linuxen, MIDIa ALSA bidez doa: zure erabiltzaileak sekuentziadorea
irekitzeko baimena izan behar du (`/dev/snd/seq`, normalean `audio`
taldean egonda).

## Kontrol bat lotzea {#binding-a-control}

Erreproduzitzaile bakoitzerako errenkada bat dago ekintza bakoitzeko:
**Play / Hurrengoa**, **Pausatu**, **Gelditu**, **Itzaltzearekin gelditu**,
**Berrabiarazi**, **Aurrekoa**, **CUE** eta **Bolumena**.

1. Egin klik errenkadako **Ikasi** botoian.
2. Sakatu nahi duzun botoia edo mugitu nahi duzun faderra (bitartean
   **Mugitu kontrol bat…** dio). Botoi batek tekla bat, pad bat edo control
   change bat bidaltzen duen botoi bat hartzen du; **Bolumena** aukerak
   fader edo biratzeko kontrol bat (control change bat) edo pitch-bend
   fader bat hartzen du.
3. Errenkadak gailua eta kontrola erakusten ditu, adibidez
   `APC mini · 36 nota, 1 kanala`.

Kontrol hori beste ekintza bati lotuta bazegoen, honetara pasatzen da.
`Esc` teklak edo botoian berriro klik egiteak ikasketa bertan behera uzten
du. **Garbitu** botoiak lotura bat kentzen du.

## Kontrolek nola jokatzen duten {#how-the-controls-behave}

- Botoi batek sakatzean eragiten du (tekla edo pad bat behera joatean, edo
  control change bat bere tartearen erditik gora igarotzean), pantailako
  erreproduzitzailearen botoiak bezala. Pantailan ilunduta dagoen botoi
  batek ez du ezer egiten.
- Fader batek bolumena mugitzen du pantailako faderraren eskala berean.
  Jauziak saihesteko, uneko bolumenera iristen denean edo hura gainditzen
  duenean bakarrik hartzen du kontrola (soft takeover); bolumena pantailan
  aldatzen bada, faderrak berriro harrapatu behar du. Ez da ezer aldatzen
  kontrol bat mugitu arte: programa abiarazteak ez du inoiz ezer airera
  bidaltzen.
- **Piztu botoiak (LED feedbacka)** aktibatuta dagoela, lotutako botoiak
  piztu egiten dira: Play erreproduzitzailea airean dagoen bitartean,
  Pausatu keinuka pausan dagoen bitartean, CUE aurrez entzuten ari den
  bitartean, eta Gelditu, Itzaltzearekin gelditu, Berrabiarazi eta Aurrekoa
  eragin dezaketen bitartean. Argiak izen bereko kontrolagailuaren irteera-
  atakara doaz; beste bat ezar daiteke `config.json` fitxategian
  (`midi.devices`).
