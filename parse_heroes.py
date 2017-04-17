import io
import json

#Helper function to get text in between to strings
def findBetween( string, before, after, start=0, end=None ):
	pos1 = string.find( before, start, end )
	if pos1 > -1:
		pos2 = string.find( after, pos1+len(before), end )
		if pos2 > -1:
			return string[ pos1+len(before) : pos2 ]
	else: return None

#Finds the position of a block enclosed in a bracket language.
#From start searches for the opening bracket, this is the starting positon
#Until it encounters a matching closing bracket, this is the end position.
#Returns None if no block was found 
def findBlock( string, start, open="{", close="}" ):
	starting_position = string.find( open, start )
	if starting_position == -1: return None
	open_brackets = 0;
	position = starting_position
	for character in string[starting_position:]:
		if character == open:
			open_brackets += 1
		elif character == close:
			open_brackets -= 1
		if open_brackets == 0:
			return (starting_position, position)
		position += 1
	return None #The block never finished

#Find a value to a key like in: "ArmorPhysical"				"-1"		
def getValue( line ):
	pos1 = line.find( '"' )
	pos2 = line.find( '"', pos1+1 )
	if pos1 == -1: return None
	key = line[pos1+1:pos2]
	value = findBetween( line, '"', '"', pos2+1 )
	return (key,value)

def readHero( string, name, start, end ):
	hero = dict();
	hero["Name"] = name
	#Keys we are interested in that will be stored as floats
	float_keys = ["ArmorPhysical",
			"MagicalResistance",
			"AttackDamageMin",
			"AttackDamageMax",
			"AttackRate",
			"AttackRange",
			"ProjectileSpeed",
			"AttributeBaseStrength",
			"AttributeStrengthGain",
			"AttributeBaseIntelligence",
			"AttributeIntelligenceGain",
			"AttributeBaseAgility"		,
			"AttributeAgilityGain",
			"MovementSpeed",
			"MovementTurnRate",
			"StatusHealth",
			"StatusHealthRegen",
			"StatusMana",
			"StatusManaRegen",
			"VisionDaytimeRange",
			"VisionNighttimeRange"]
	string_keys = ["AttributePrimary", "AttackCapabilities"] #keys that will be stored as strings
	for line in string[start:end].split("\n"):
		result = getValue( line )
		if result == None: continue
		(key,value) = result
		#print("Key:", key, "Value:", value)
		if key in float_keys:
			if value == "": value = 0.0 #Sometimes for example missilespeed is set to "" on melee heroes 
			hero[key] = float(value)
		elif key in string_keys:
			hero[key] = value
	return hero

def readItem( string, name, start, end ):
	item = dict()
	item["Name"] = name
	# Values in the same row mean that the effects are related
	# TODO: Lifesteals: "vampiric_aura", 
	float_keys = ["ItemCost",
				"bonus_damage", "damage_aura",
				"bonus_armor", "armor_aura", "aura_bonus_armor", "aura_positive_armor", "aura_armor",
				"bonus_health_regen", "bonus_regen", "aura_health_regen", "hp_regen", "health_regen",
				"health_regen_rate", #Relative hp regen, Heart = 0.02
				"mana_regen_aura", "aura_mana_regen", #absolute mana regen
				"bonus_mana_regen", "mana_regen", "bonus_mana_regen_pct", #relative mana regen as percentage (sobi mask = 50)
				"bonus_health",
				"bonus_mana",
				"bonus_chance",	"bonus_chance_damage", #like javelin
				"bonus_attack_speed", "bonus_speed", "aura_attack_speed", "bonus_aura_attack_speed_pct",
				"crit_chance", "crit_multiplier",
				"bash_chance", "bash_chance_melee",	"bash_chance_ranged", "bash_damage",
				"damage_block_melee", "damage_block_ranged", "block_chance",
				"bonus_evasion",
				"bonus_agility",
				"bonus_intellect", "bonus_intelligence",
				"bonus_strength",
				"bonus_all_stats", "bonus_stats",
				"bonus_spell_resist", "magic_resistance", "bonus_magical_armor", #as percentage, hood = 30
				"bonus_movement_speed", "bonus_movement", #sometimes relative as percentage(manta,...), sometimes absolute (euls, boots)
				"movement_speed_percent_bonus", "bonus_aura_movement_speed_pct", #always relative as percentage
				"cleave_damage_percent", "cleave_radius",
				"lifesteal_percent"]
	for line in string[start:end].split("\n"):
		result = getValue( line )
		if result == None: continue
		(key,value) = result
		#print("Key:", key, "Value:", value)
		if key in float_keys:
			try:
				item[key] = float(value)
			except ValueError: pass #happens for items like Dagon that use level syntax
					
	return item

#Parse heroes in valve's npc_heroes.txt located in the dota2 vpk under /scripts/npc/
#and emit json for every hero
def parseHeroes():
	text = None
	heroes = []

	with open('data/npc_heroes.txt', 'r', encoding='utf-8') as f:
		text = f.read()

	pos = text.find( "// HERO: Base" )
	(start, end) = findBlock( text, pos )
	heroes.append( readHero( text, "Base", start, end ) )

	name = findBetween( text, "// HERO: ", "\n", end )
	while name != None:
		print("Parsed", name)
		(start, end) = findBlock( text, end )
		heroes.append( readHero( text, name, start, end ) )
		#print(hero)
		#print(json.dumps(hero, sort_keys=True, indent=4))
		name = findBetween( text, "// HERO: ", "\n", end )

	for h in heroes:
		with open('data/json/heroes/'+h["Name"]+".json", 'w', encoding='utf-8') as f:
			json.dump(h, f, ensure_ascii=False, sort_keys=True, indent=4)

#Parse items in valve's items.txt located in the dota2 vpk under /scripts/npc/
#and emit json for every item
def parseItems():
	text = None
	items = []

	with open('data/items.txt', 'r', encoding='utf-8') as f:
		text = f.read()

	beforeItem = "\t//=================================================================================================================\n\t// "
	pos = text.find( beforeItem )
	name = findBetween( text, "// ", "\n", pos )
	while name != None:
		print("Parsed", name)
		(start, end) = findBlock( text, pos )
		if not name.startswith("Recipe: "):
			item = readItem( text, name, start, end )
			if len(item) > 2 and "ItemCost" in item and item["ItemCost"] != 0: #Filter some items like "Greevil Blink Dagger" or items we could not parse anything about
				items.append( item )
		#print(json.dumps(item, sort_keys=True, indent=4))
		pos = text.find( beforeItem, end )
		name = findBetween( text, "// ", "\n", pos )
	for i in items:
		with open('data/json/items/'+i["Name"]+".json", 'w', encoding='utf-8') as f:
			json.dump(i, f, ensure_ascii=False, sort_keys=True, indent=4)
	
parseHeroes()
parseItems()