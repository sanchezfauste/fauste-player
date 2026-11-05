# Kartutxo-panela

Kartutxo-panela erreproduzitzaileen azpiko botoi-zerrenda da, edo haien
ondoan dagoena (ikus [Diseinua](layout.md#move-and-resize-the-cartwall)).
Botoi bakoitzak, **kartutxo** batek, soinu bat jotzen du berehala: jingleak,
efektuak, iragarkiak. Kartutxoak beren irteeretan jotzen dira,
erreproduzitzaileekiko independenteki.

![Kartutxo-panela, kartutxo bat jotzen ari dela](../../images/guide/cartwall.png)

## Erabilera {#using-it}

- **Egin klik kartutxo batean** jaurtitzeko. **Egin klik berriro**
  gelditzeko.
- **Arrastatu pistak [liburutegiaren leihotik](library.md)** kartutxo batera
  lehena bertan jartzeko (kartutxoa geldituta dagoenean, bere fitxategia
  ordezten da). Jotzen ari den edo CUEan dagoen kartutxo batek jaregitea
  baztertzen du.
- **Gelditu dena** botoiak (barraren eskuineko muturrean) jotzen ari diren
  kartutxo guztiak gelditzen ditu, orri guztietan. Bere etiketak zenbat
  ari diren jotzen erakusten du, **Gelditu dena (2)** bezala; bat ere ez
  dagoenean ilunduta dago eta ez du zenbakirik erakusten.
- Kartutxo bat jotzen ari den bitartean, bere ertzak eta ezkerretik jotzen
  den heinean hazten den barrak kartutxoaren [kolorea](#setting-up-carts)
  hartzen dute (gorria, koloreik ez duen kartutxo batean), marra mehe batek
  jotako ertza markatzen du uhin-formaren gainean, eta bere denborak atzerako
  kontaketa egiten du. Marra leunki mugitzen da, kartutxo labur batean ere.
- Kartutxo baten fitxategia aztertu ondoren, bere uhin-forma argi-argi
  agertzen da izenaren atzean, berehala jotzen duen soinu bat poliki-poliki
  sartzen denetik bereizteko. Kartutxoak jotzen duena biltzen du, cue-in
  puntutik cue-out punturaino.
- Kartutxoak lehenespenez **gainjartzen** dira: bigarren bat jaurtitzeak ez
  du lehena gelditzen. **Gelditu beste kartutxoak jaurtitzean** aukerarekin
  konfiguratutako kartutxo batek airean dagoen beste kartutxo guztiak
  gelditzen ditu lehenik, edozein orritan.
- **Begizta** aukerarekin konfiguratutako kartutxo bat bere cue-in puntutik
  hasten da berriro amaierara iristean, tarterik gabe, gelditu arte.
- **Egin eskuineko klik** kartutxo batean aukera gehiagorako:

| Elementua | Ekintza |
|---|---|
| Aurrez entzun CUEan | Kartutxo-panelaren CUE irteeran jotzen du (ilunduta, kartutxo-panelak Cue irteerarik ez duenean). Aireko irteeran, han zerbait jotzen ari den bitartean, lehenik galdetzen du (ikus [CUE](players.md#cue-pre-listen)) |
| Gelditu | Gelditzen du |
| Editatu… | Ezarpenetan irekitzen du |

Kartutxo huts baten menuak **Editatu…** bakarrik du, bere fitxategia
aukeratzeko.

- **Orriak:** **KARTUTXOAK** ondoko fitxek orriz aldatzen dute. Puntu gorri
  batek orri horretako kartutxo bat jotzen ari dela adierazten du.
- Egin klik **KARTUTXOAK** aukeran kartutxo-panela tolestu edo berriro
  zabaltzeko. Erreproduzitzaileen ondoan tolestuta, zerrenda estu bat da,
  zabaltzeko gezi batekin eta, kartutxo bat jotzen ari den bitartean, puntu
  gorri batekin.
- Erreproduzitzaileen azpian, bere errenkadek behar duten altuerakoa da, eta
  leihoa baxua denean botoiak txikitu egiten dira (gutxieneko altuera
  bateraino), konfiguratutako errenkada guztiak sar daitezen. Kartutxo-panela
  tamainaz aldatu ondoren, edo erreproduzitzaileen ondoan, botoiak hazten dira
  hura betetzeko, eta izenek eta denborek beren tamaina mantentzen dute
  botoi bakoitzaren erdian. Edonola ere, kartutxo-panelak botoirik txikienak
  ere sartzen ez direnean bakarrik korritzen du.

| Botoiaren itxura | Esanahia |
|---|---|
| Puntu morea | Jinglea |
| Puntu anbarra | Efektua |
| Puntu grisa | Iragarkia (komertziala) |
| Ertz, tonu eta puntu koloreztatua | Kartutxoaren kolorea, Ezarpenetan aukeratua: puntuak motaren kolorea ordez kolore hori hartzen du |
| ↻ motaren ondoren | Begizta egiten du |
| ✋ motaren ondoren | Beste kartutxoak gelditzen ditu jaurtitzean |
| Gurutze / abisu-ikurra duen fitxategia | Fitxategia falta da / ezin da deskodetu; pasatu sagua kartutxoaren gainetik arrazoia eta bide-izena ikusteko. Falta den fitxategi bat berriro bilatzen da 30 s-ro (`tuning.missing_recheck_ms`). |
| "Hutsik", ilunduta | Ez dago fitxategirik esleituta |

Kartutxoek pisten markatzaile berberak erabiltzen dituzte. Beren cue-in
puntuan hasten dira eta beren cue-out puntuan amaitzen, eta horiek
erreproduzitzaile baten uhin-forman edita ditzakezu fitxategia bertan
kargatuta dagoenean.

## Teklatua {#keyboard}

Lehenespenez, **F1**…**F12** teklek erakutsitako orriko 1–12 kartutxoak
jaurtitzen dituzte, eta **Ctrl+Space** konbinazioak kartutxo guztiak
gelditzen ditu (**Gelditu dena** botoiaren berdina; CUEan aurrez entzuten ari
zaren kartutxo bat ere gelditzen du, kartutxorik jotzen ari ez denean ere).
Ikus [Teklatua](keyboard.md) aldatzeko.

Kartutxo-panela tolestu edo zabaltzen duen laster-teklak ez du ezer egiten
kartutxo-panela goiko barratik ezkutatuta dagoen bitartean (ikus
[Diseinua](layout.md#show-and-hide-panels)).

## Kartutxoak konfiguratzea {#setting-up-carts}

Joan **Ezarpenak → Kartutxoak** atalera:

- **Orriak:** sortu, aldatu izena, ezabatu (azken orria ezin da ezabatu),
  eta ezarri saretaren tamaina (errenkadak × zutabeak). Sareta txikiago
  bat baztertu egiten da fitxategia duten kartutxoak galduko balira.
  Graduatzaile baten ondoko koadroan idatzitako tamaina Enter sakatzean edo
  koadrotik irtetean aplikatzen da; Escape-k bertan behera uzten du.
- **Inportatu… / Esportatu…** botoiek orri bat `.cartpage.json` fitxategi
  batean gordetzen dute eta berriro kargatzen dute, adibidez estudioen
  artean partekatzeko. Fitxategien bide-izen erlatiboak fitxategiaren
  karpetaren arabera ebazten dira. Koloreak orriarekin batera doaz; bertsio
  zaharrago bateko fitxategi bat kolorerik gabe inportatzen da.
- **Kartutxoak:** egin klik saretako kartutxo batean, eta ezarri bere
  izena, fitxategia (**Aukeratu…** edo **Kendu**), mota, **Kolorea**,
  **Begizta** eta **Gelditu beste kartutxoak jaurtitzean**. **Kolorea**
  aukerak kolorerik ez eta zortzi kolore eskaintzen ditu (gorria, laranja,
  anbarra, berdea, turkesa, urdina, morea, arrosa): eman kolore bat
  iragarki guztiei eta beste bat estazioaren identifikatzaileei, begirada
  batean aurkitzeko. Kartutxo huts batek bere kolorea mantentzen du, eta
  kartutxo baten fitxategia kentzeak ez du kolorea kentzen. Fitxategia duen
  kartutxo batek **Normalizaziotik kanpo utzi** eta **Doikuntza** ere baditu,
  bere pistaren [ozentasun-normalizazioaren](loudness.md#per-track-exclude-and-trim)
  ezarpenak, neurketa etiketaren azpian dagoela.

Kartutxo-panelaren beraren Main eta Cue irteerak **Ezarpenak →
Audio-irteerak** atalean daude (**Kartutxo-panela** errenkada).
