# Kartutxo-panela

Kartutxo-panela erreproduzitzaileen azpiko botoi-zerrenda da. Botoi
bakoitzak, **kartutxo** batek, soinu bat jotzen du berehala: jingleak,
efektuak, iragarkiak. Kartutxoak beren irteeretan jotzen dira,
erreproduzitzaileekiko independenteki.

![Kartutxo-panela, kartutxo bat jotzen ari dela](../../images/guide/cartwall.png)

## Erabilera {#using-it}

- **Egin klik kartutxo batean** jaurtitzeko. **Egin klik berriro**
  gelditzeko.
- **Gelditu dena** botoiak (barraren eskuineko muturrean) jotzen ari diren
  kartutxo guztiak gelditzen ditu, orri guztietan. Bere etiketak zenbat
  ari diren jotzen erakusten du, **Gelditu dena (2)** bezala; bat ere ez
  dagoenean ilunduta dago eta ez du zenbakirik erakusten.
- Kartutxo bat jotzen ari den bitartean, bere ertza gorri jartzen da, barra
  gorri bat txikitzen doa jotzen den heinean, eta bere denborak atzerako
  kontaketa egiten du.
- Kartutxoak lehenespenez **gainjartzen** dira: bigarren bat jaurtitzeak ez
  du lehena gelditzen. **Gelditu beste kartutxoak jaurtitzean** aukerarekin
  konfiguratutako kartutxo batek airean dagoen beste kartutxo guztiak
  gelditzen ditu lehenik, edozein orritan.
- **Begizta** aukerarekin konfiguratutako kartutxo bat bere cue-in puntutik
  hasten da berriro amaierara iristean, tarterik gabe, gelditu arte.
- **Egin eskuineko klik** kartutxo batean aukera gehiagorako:

| Elementua | Ekintza |
|---|---|
| Aurrez entzun CUEan | Kartutxo-panelaren CUE irteeran jotzen du (ilunduta, kartutxo-panelak bere Main irteeraz bestelako Cue irteerarik ez duenean) |
| Gelditu | Gelditzen du |
| Editatu… | Ezarpenetan irekitzen du |

Kartutxo huts baten menuak **Editatu…** bakarrik du, bere fitxategia
aukeratzeko.

- **Orriak:** **KARTUTXOAK** ondoko fitxek orriz aldatzen dute. Puntu gorri
  batek orri horretako kartutxo bat jotzen ari dela adierazten du.
- Egin klik **KARTUTXOAK** aukeran zerrenda tolestu edo berriro zabaltzeko.
- Leihoa baxua denean, botoiak txikitu egiten dira (gutxieneko altuera
  bateraino), konfiguratutako errenkada guztiak sar daitezen; kartutxo-
  panelak botoirik txikienak ere sartzen ez direnean bakarrik korritzen du.

| Botoiaren itxura | Esanahia |
|---|---|
| Puntu morea | Jinglea |
| Puntu anbarra | Efektua |
| Puntu grisa | Iragarkia (komertziala) |
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

## Kartutxoak konfiguratzea {#setting-up-carts}

Joan **Ezarpenak → Kartutxoak** atalera:

- **Orriak:** sortu, aldatu izena, ezabatu (azken orria ezin da ezabatu),
  eta ezarri saretaren tamaina (errenkadak × zutabeak). Sareta txikiago
  bat baztertu egiten da fitxategia duten kartutxoak galduko balira.
- **Inportatu… / Esportatu…** botoiek orri bat `.cartpage.json` fitxategi
  batean gordetzen dute eta berriro kargatzen dute, adibidez estudioen
  artean partekatzeko. Fitxategien bide-izen erlatiboak fitxategiaren
  karpetaren arabera ebazten dira.
- **Kartutxoak:** egin klik saretako kartutxo batean, eta ezarri bere
  izena, fitxategia (**Aukeratu…** edo **Kendu**), mota, **Begizta** eta
  **Gelditu beste kartutxoak jaurtitzean**.

Kartutxo-panelaren beraren Main eta Cue irteerak **Ezarpenak →
Audio-irteerak** atalean daude (**Kartutxo-panela** errenkada).
