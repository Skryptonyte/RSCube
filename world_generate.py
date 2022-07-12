
import gzip
import threading
import struct
import random

        
def generateFlatWorld(fileName,worldX, worldY, worldZ):
    print("Generating new flat world at", fileName)
    volume = worldX*worldY*worldZ
    blocksArray = bytearray(volume)
    
    f = gzip.open(fileName,"wb+")
    
    f.write(struct.pack('h',1874))
    
    f.write(struct.pack('h',worldX))
    f.write(struct.pack('h',worldZ))
    f.write(struct.pack('h',worldY))
    
    
    spawnX = worldX >> 1
    spawnY = (worldY >> 1) + 1
    spawnZ = worldZ >> 1
    f.write(struct.pack('h',spawnX))
    f.write(struct.pack('h',spawnZ))
    f.write(struct.pack('h',spawnY))
    
    f.write(struct.pack('B',0))
    f.write(struct.pack('B',0))
    
    f.write(struct.pack('B',0))
    f.write(struct.pack('B',0))
    
    maxY = worldY >> 1
    
    print("Calculating")
    for y in range(maxY):
        for x in range(worldX):
            for z in range(worldZ):
                
                calculatedIndex = x + worldX*(z+worldZ*y)
                
                if (y == maxY-1):
                    blocksArray[calculatedIndex] = 2
                else:
                    blocksArray[calculatedIndex] = 3
                
                
    print("Writing to file")
    f.write(blocksArray)
        
    f.close()
    print("Finished")
    

# EXPERIMENTAL
import opensimplex
def noise2D(x,z):
    return 0.5*(opensimplex.noise2(x,z) + 1.0)

def generateSimplexWorld(fileName,worldX, worldY, worldZ):
    print("Generating new procedural world at", fileName)
    volume = worldX*worldY*worldZ
    blocksArray = bytearray(volume)
    
    print("Size: ",worldX, worldY, worldZ)
    f = gzip.open(fileName,"wb+")
    
    f.write(struct.pack('h',1874))
    
    f.write(struct.pack('h',worldX))
    f.write(struct.pack('h',worldZ))
    f.write(struct.pack('h',worldY))
    
    
    spawnX = worldX >> 1
    spawnY = (worldY) - 1
    spawnZ = worldZ >> 1
    
    print(spawnX, spawnY, spawnZ)
    f.write(struct.pack('h',spawnX))
    f.write(struct.pack('h',spawnZ))
    f.write(struct.pack('h',spawnY))
    
    f.write(struct.pack('B',0))
    f.write(struct.pack('B',0))
    
    f.write(struct.pack('B',0))
    f.write(struct.pack('B',0))
    
    maxY = worldY >> 1
    
    print("Calculating")

    opensimplex.seed(random.randint(0,2**64))
    for x in range(worldX):
        for z in range(worldZ):
            
            amp = 0.0
            octaveCount = 20
            currentAmp = 1.0
            freq = 10
            n = 0
            for o in range(octaveCount):
                n += currentAmp*noise2D(freq*x/worldX,freq*z/worldZ)
                amp += currentAmp
                freq *= 2
                currentAmp *= 0.5
            n = n / amp
            #n = n ** 2
            maxY = (worldY >> 1) + int(n*( (worldY>>2) ))

            for y in range(maxY):
                calculatedIndex = x + worldX*(z+worldZ*y)
                if (y == maxY-1):
                    blocksArray[calculatedIndex] = 2
                elif y < maxY - 5:
                    blocksArray[calculatedIndex] = 1
                else:
                    blocksArray[calculatedIndex] = 3     
                    
    print("Writing to file")
    f.write(blocksArray)
        
    f.close()
    print("Finished")


if __name__ == "__main__":

    world_type = input("Enter type (flat/terrain): ")
    worldName = "world"
    print("Enter sizes: ")
    x = int(input("Enter x: "))
    y = int(input("Enter y: "))
    z = int(input("Enter z: "))

    if world_type == "flat":
        generateFlatWorld("maps/"+worldName+".lvl",x,y,z)
    elif world_type == "terrain":
	    generateSimplexWorld("maps/"+worldName+".lvl",x,y,z)




