package catalog

import "math/rand/v2"

// FullNames contains 500 fantasy names for deterministic patron generation.
// Randomly sampled from a pool of ~6000 names to ensure diversity across races.
var FullNames = []string{
	"Abigail Gormless", "Aelia Nightrunner", "Aemilia Hobbleknee", "Aeranduil Melorian", "Aerandur Cirdanor",
	"Aldaron Maegdor", "Aldith Jadestar", "Alice McCafferty", "Alric Tinderwick", "Ambrose Thiel",
	"Ambrose Wieland", "Amgar Thunderstorm", "Andrella Barleyfoot", "Angrim Dragonheart", "Angrod Cirdanor",
	"Angrvadis Keeneye", "Annie Jackman", "Antoinette Hookum", "Arabella Newcomb", "Aranarth Celeborn",
	"Arandur Elgarion", "Arasinya Ilendil", "Archibald Dobson", "Ardon Ironwind", "Arula Auburnleaf",
	"Arula Shepherd", "Augusta Crickhollow", "Aulendil Caerin", "Avendil Ardalas", "Balrik Firestrike",
	"Balrik Goldenstrike", "Balthazar Eckert", "Balthazar Haas", "Balwin Littleberry", "Bard Lindor",
	"Bardin Moltenheart", "Beldon Brightmeadow", "Beldrith Honorforged", "Belle Haverstock", "Bellona Chestnut",
	"Benedict Acker", "Benedict Gebhardt", "Benedict Morley", "Benjamin Walker", "Beren Rhovendor",
	"Bernadette Vance", "Borgula Skul'gron", "Brannar Rocksplitter", "Brannar Thunderforge", "Brumley Woodruff",
	"Brynva Glimmerbeard", "Bungo Bottlehill", "Buzgash Urz'gol", "Caius Schneider", "Calion Enethril",
	"Camilla Oakentoe", "Cassandra Schmitz", "Cassia Willowhollow", "Cassius Conrad", "Cecilia Wheatgrass",
	"Celebeth Caerion", "Celedorn Enyandil", "Celestia Brandt", "Celestia Conrad", "Celestia Gebhardt",
	"Celestria Elgarion", "Celia Millar", "Charles Northend", "Cheribelle Peachblossom", "Clara King",
	"Clement Sunninghill", "Cornelia Urquhart", "Cotton Kettlewhistle", "Cotton Mossybrook", "Cressida Beyerlein",
	"Curufin Saldorion", "Dagna Pinebranch", "Dagnis Rockcrusher", "Dandy Applegrove", "Danoviel Lithilas",
	"Daphne Bagshot", "Daphne Thistle", "Dargur Pineheart", "Darien Haladir", "Delin Runehammer",
	"Diana Hobblefoot", "Disja Oathsworn", "Dolvi Ashenshield", "Dolvi Ironhelm", "Domitia Harvestmoon",
	"Donnegan Candlewick", "Donnegan Talltree", "Dorga Bor'ruk", "Dorga Rok'gron", "Dorgha Drek'ka",
	"Dorgoth Dush'nak", "Dorgrim Lorewood", "Dorgula Gol'mok", "Dorgula Skul'gron", "Dorian Holzknecht",
	"Drar Jewelstrike", "Drenna Earthwhisper", "Drenna Goldenshaper", "Durgash Gol'mok", "Durin Jadegrove",
	"Dushgol Gol'dur", "Dushgor Gol'burz", "Ecthelion Aralion", "Ecthelion Celeborn", "Edgar Stooge",
	"Edith Chapman", "Edmund Thiele", "Eirlys Nutmeg", "Eladion Galdorin", "Elara Zumwinkel",
	"Eldor Bitterpick", "Eldor Glowforge", "Eldor Lionroar", "Elendil Aralion", "Elissia Elnaril",
	"Elissia Idhrendil", "Elithil Acalor", "Elladan Valandorin", "Elma Whitewind", "Elrond Celador",
	"Elrond Whisperingwind", "Elwis Bluebell", "Emeric Baumann", "Emerwen Acalor", "Emma Shacklebolt",
	"Erestor Arnoron", "Estella Knibbs", "Evaine Schmid", "Evangeline Huber", "Faelion Orandor",
	"Faelwen Lithariel", "Fairwen Astoron", "Faladon Carnadon", "Faladon Falathan", "Faustina Hobblefoot",
	"Faylinn Cinderfoot", "Felicity Orlando", "Findegil Haladan", "Fiona Fisher", "Fiona Frey",
	"Fiora Lechner", "Fortuna Treetop", "Fosco Frostwhisper", "Franklin Romilda", "Frederica Skower",
	"Frederick Krause", "Frederick Lancaster", "Frodin Starkstone", "Gabriella Friar", "Gabriella Podmore",
	"Galadhrim Lorien", "Gandalf Glimmerbeard", "Gawain Zimmermann", "Genevieve Zahn", "Gerard Krug",
	"Gerard Kunz", "Ghashnakh Drak'thul", "Ghazga Mog'dor", "Ghorga Gash'nak", "Ghorga Nar'gul",
	"Ghorga Zog'dor", "Gimrash Dor'gon", "Ginevra Eisenmann", "Giselle Lang", "Glira Pinetide",
	"Glorfindel Rhovendor", "Golgakh Thrag'ash", "Golgakh Vor'gol", "Golnaz Drek'gash", "Golug Lug'gor",
	"Gorbag Mog'ash", "Gorblud Skul'mog", "Gord Mistyspring", "Gorgash Mog'ash", "Gorgol Rok'gron",
	"Gorgrak Rok'gron", "Gorka Drok'mog", "Gorka Urz'gol", "Gorka Vor'gol", "Gorluk Drek'gash",
	"Goror Deepdelver", "Gorzug Thrak'gor", "Gothmog Dor'gon", "Grazha Drek'ka", "Grerda Wildwind",
	"Grishna Mok'gar", "Grogula Thruk'dor", "Grorin Firestrike", "Grugula Gol'burz", "Grumash Mog'lok",
	"Grushnakh Skul'mog", "Gundin Blackvein", "Gwindor Nimrion", "Halda Pinevein", "Halfling Thorondil",
	"Halfling Zephiroth", "Halson Tricklebrook", "Hamfa Dragonforge", "Hamfast Rosepetal", "Hamson Applegrove",
	"Harin Thunderforge", "Harinor Firestrike", "Harinor Jewelsmith", "Harmony Strawberry", "Harold Wagner",
	"Hector Gebhardt", "Henry Tattle", "Hob Peachblossom", "Hobson Ripplebrook", "Hofar Mountainsmith",
	"Horatio Conrad", "Hornblower Larkspur", "Hornblower Redcheek", "Hornblower Tinypocket", "Hugo Mushy",
	"Hulna Bitterpick", "Hurin Lorehammer", "Hurin Stonewhisper", "Idrilas Tirithorn", "Ignatius Knecht",
	"Ignatius Schwarz", "Imara Mithrilhelm", "Imelda Fischer", "Imelda Rohde", "Imrahil Valandor",
	"Inga Rockjaw", "Ingva Amberheart", "Irdra Longstride", "Irisa Fallowfield", "Ironar Honedblade",
	"Ironin Wildstone", "Isaiah Noggin", "Isembard Hobblefoot", "Ithildin Zimrathor", "Ithilion Boromir",
	"Ivy Harrow", "Jane Roscoe", "Jeremiah Herzog", "Jeremiah Rapp", "Jeremiah Schott",
	"Jessamine Zuber", "Jewel Tricklebrook", "Jocelyn Gebhardt", "Jocelyn Zink", "John Mush",
	"Judith Dorkins", "Julia Lupus", "Juliet Nixon", "Justina Silverbell", "Kagrim Earthshaper",
	"Katherine Aragon", "Kenneth Wallace", "Kessa Frostbranch", "Kilin Honorstone", "Kilin Kingsong",
	"Kragula Grom'gul", "Krazha Gol'mok", "Lagduf Skul'gron", "Lagduf Ur'burz", "Largo Skiprock",
	"Laurentia Mossleaf", "Lavender Brightmeadow", "Lavinia Beyer", "Lavinia Franke", "Lavinia Mudblood",
	"Lenora Foy", "Leonora Wilkes", "Leopold Innes", "Leticia Dornton", "Leticia Dudley",
	"Lobelia Badgerhill", "Lobelia Copperknob", "Lofar Axebringer", "Lorelei Dewdrop", "Lorin Yewmane",
	"Lotho Primrose", "Lyra Wildstone", "Lysander Schwab", "Maebry Cobblehill", "Maebry Sandybanks",
	"Maelisande Krause", "Maelisande Sonnemann", "Maendir Saldorion", "Magnus Maier", "Magnus Schulz",
	"Magnus Schumacher", "Marcella Tumblestone", "Marcia Springdale", "Marigold Schubert", "Marni Ironwind",
	"Mauhur Vor'gol", "Mauzgash Raz'nak", "Maximilian Stolz", "May Warbeck", "Maznash Zog'dor",
	"Meril Elnarion", "Merrick Klein", "Mirella Eggert", "Mogdush Drek'gash", "Morgak Bor'ruk",
	"Morgana Barleyfoot", "Morgana Pebblebrook", "Morwen Haladir", "Morwenna Albrecht", "Mugash Drek'gash",
	"Mugha Zogdu", "Muzgha Mol'dok", "Nagha Gor'ak", "Nain Glimmerbeard", "Nashga Grom'gul",
	"Nashgrodha Zog'gar", "Nashka Gor'ak", "Nazhga Drak'thul", "Nellie Burbage", "Nellie Landsman",
	"Nenloth Astoron", "Nerissa Weber", "Nerissa Zorn", "Nessa Blackmantle", "Nessa Hazelnut",
	"Nessa Lilybrook", "Nibs Hobblefoot", "Nibs Wheatgrass", "Nidra Ashenmantle", "Nidra Gemsunderer",
	"Nigel Zamojski", "Nirneth Narothor", "Nistra Mountainguard", "Nora McNamara", "Nora Oxfordshire",
	"Nordin Proudbeard", "Norva Jadegrip", "Nyra Deepforge", "Octavius Wimpole", "Odette Onions",
	"Odette Webster", "Odfri Jewelsmith", "Odna Ironwind", "Oliver Sidwell", "Olo Frostwhisper",
	"Olo Pebblebrook", "Olva Deepdelve", "Oscar Weasley", "Pearl Morlock", "Pearl Spencer",
	"Penelope McNair", "Percival Galatea", "Peregrin Wildflower", "Peregrine Neubauer", "Persephone Lindemann",
	"Pervinca Cobblehill", "Pervinca Woolysocks", "Peter Starling", "Pippin Copperfield", "Posco Shepherd",
	"Quirinus Zorn", "Ragnar Honorshaper", "Ragnor Mountainguard", "Raina Thunderfoot", "Ralgar Jewelsmith",
	"Ralion Ariandel", "Raphael Fink", "Raphael Ziegler", "Rindon Ariandel", "Roderic Driftwood",
	"Roper Tumbleweed", "Rosalind Hammer", "Rosalind Meyer", "Rosalind Schneider", "Rosalind Talbot",
	"Rowan Quicksilver", "Rowena Angermann", "Rowena McCluck", "Ruddy Riverstone", "Rupert Binns",
	"Sabrina MacNair", "Saeldil Belarond", "Saeros Glorinor", "Samuel Grimsditch", "Samuel Melling",
	"Sebastian Thicknesse", "Sebastian Winkler", "Selene Fuchs", "Septima Starflower", "Seraphina Nimblefingers",
	"Seraphina Oakentoe", "Serena Chestnut", "Shagdor Gash'nak", "Shagdush Mol'dok", "Shagnaz Mok'gar",
	"Shargash Dor'gon", "Shargash Dush'nak", "Shazga Gor'mok", "Sigismond Tinypocket", "Silas Terry",
	"Snaga Thrak'gor", "Snagha Gol'burz", "Snagha Thrak'gor", "Snagha Zur'kosh", "Snagula Drok'gar",
	"Snargash Rok'mog", "Snash Mog'ash", "Snazha Drek'gash", "Sofra Dragonheart", "Sophia Wurts",
	"Sylvana Kopp", "Synva Jadebeard", "Tabitha Ozanne", "Tansy Woodchuck", "Tarja Bouldershield",
	"Tarja Earthward", "Tatiana Schmid", "Tertulla Tinderwick", "Thaddeus Hubert", "Tharin Gemtamer",
	"Theodore Jackman", "Thokka Gol'burz", "Thokka Skul'rok", "Thomas Dudley", "Thomas Roberts",
	"Thora Flameforge", "Thorin Zimranor", "Throkgula Mog'ash", "Throkha Zog'gar", "Timothy Worpel",
	"Tindra Jadehelm", "Titania Conrad", "Titania Jung", "Torin Mountainstone", "Tubbo Hollyberry",
	"Tubbo Springdale", "Tulip Tinypocket", "Ugga Drok'gar", "Uglog Gol'burz", "Ulfgar Runebeard",
	"Ulric Aragon", "Ulric Gwenog", "Ulysses Witt", "Ungoliant Borondir", "Uriah Huber",
	"Ursula Pfeiffer", "Ursula Schumacher", "Uruk Drek'gash", "Urzogga Darg'gash", "Ushnag Narg'hul",
	"Uzgash Dush'nak", "Valandil Rhovendor", "Valandur Arandor", "Valdra Emberbeard", "Vanda Blackstone",
	"Varag Keenaxe", "Varion Rhovendur", "Veigdis Ashenmantle", "Veigdis Frostbeard", "Verina Thistletop",
	"Vibia Sandybanks", "Victor Lechner", "Victor Zander", "Vilda Earthwhisper", "Vilda Firestrike",
	"Vilda Wildward", "Vincent Langley", "Viola Stoogis", "Wilcome Picklefoot", "Wilcome Whisperingwind",
	"Wilcome Whisperwind", "Wilfred Martin", "Wilfred Sommer", "Wilwarin Strawberry", "Winona Moran",
	"Winston Jugson", "Winston Manco", "Winston McDowell", "Wiseacre Shadetree", "Xander Hillyard",
	"Xander North", "Xanthe Wolf", "Xavier Lehmann", "Yamin Stoutbeard", "Yapper Spiderwick",
	"Yara Woodchuck", "Yvette Hoppe", "Zacharias Brinkmann", "Zam Thistlebrook", "Zangrim Yewheart",
	"Zashga Darg'gash", "Zeff Sandybanks", "Zephaniah Stoogis", "Zephyr Schindler", "Zinnia Hedgegrove",
	"Zogga Vor'gol", "Zogzha Drek'ka", "Zugla Skul'gron", "Zugluk Zog'gar", "Zuzha Gol'dur",
}

// NamePool provides names without replacement. When the pool is exhausted,
// it reshuffles and starts over, so duplicates only occur when more names are
// needed than the pool contains.
type NamePool struct {
	names []string
	pos   int
	rng   *rand.Rand
}

// NewNamePool creates a shuffled copy of the full name list.
func NewNamePool(rng *rand.Rand) *NamePool {
	names := make([]string, len(FullNames))
	copy(names, FullNames)
	rng.Shuffle(len(names), func(i, j int) {
		names[i], names[j] = names[j], names[i]
	})
	return &NamePool{names: names, pos: 0, rng: rng}
}

// Next returns the next name from the pool, reshuffling when exhausted.
func (p *NamePool) Next() string {
	if p.pos >= len(p.names) {
		p.rng.Shuffle(len(p.names), func(i, j int) {
			p.names[i], p.names[j] = p.names[j], p.names[i]
		})
		p.pos = 0
	}
	name := p.names[p.pos]
	p.pos++
	return name
}

// GenerateName returns a random full name from the name pool.
// Deprecated: use NamePool for duplicate-free assignment.
func GenerateName(rng *rand.Rand) string {
	return FullNames[rng.IntN(len(FullNames))]
}
