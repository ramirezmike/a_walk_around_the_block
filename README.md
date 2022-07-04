# A Walk Around The Block

An entry for the [2nd Rusty game jam][jam] following the theme of [this image of a chicken/dog][theme] "interpreted however you'd like." It was made in one week using the wonderful [Bevy game engine][bevy]. 

You can play the game [here][itch].


https://user-images.githubusercontent.com/1421719/177226574-b8fc57da-490b-4ae4-8652-d31c89992b56.mp4


Other submissions for the jam can be seen [here][submissions]

A Walk Around The Block is an isometric singleplayer action game where you try to get the highest score before time runs out. Points can be added or deducted by various actions you or your pets do. Each additional pet you're walking multiplies the number of points you get per yellow orb.

There are three types of pets: Dogs, Chickens and ChickenDogs, which each have their own behaviors and interactions with the environment.

The player can walk up to 4 pets at a time, each controlled by one of the "face buttons" on your controller or the keys IJKL on your keyboard. The leashes' color corresponds to the button pressed to control that pet. Players can hold down the button to keep a constant pull on the leash or tap the button to give the leash a yank to quickly pull a pet toward the player.

Walking around the block you'll encounter neighbors, chipmunks and worms. Dogs love to be petted, but will try to chase down chipmunks. Chickens ignore the chipmunks, will eat worms and will annoy people. ChickenDogs will destroy anything they encounter.

The player levels up after each 1000 points which affects how quickly you can move and how powerful your pets are. Try to keep track of your pets because if they wander too far you may lose them and get a Game Over!

Check out my other games [here][othergames]. Also, I'm always hanging out in the [bevy discord][bevy-discord], definitely feel free to @ramirezmike me and ask questions or criticize me :)


# Running the Game

To run the game locally

```
cargo r --features bevy/dynamic
```

[jam]: https://itch.io/jam/rusty-jam-2
[bevy]: https://bevyengine.org/
[theme]: https://img.itch.zone/aW1nLzkyMjkxOTIucG5n/original/xgeODP.png 
[itch]: https://ramirezmike2.itch.io/a-walk-around-the-block 
[submissions]: https://itch.io/jam/rusty-jam-2/entries
[othergames]: https://ramirezmike2.itch.io/
[bevy-discord]: https://discord.gg/bevy
