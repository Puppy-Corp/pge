import { Cam, Camera, KeyboardEvent, Scene, Window } from "./pge";

const camera = new Camera()

class Player {
	public node: Node
	
	constructor() {
		this.node = new Node()
	}
}

const scene = new Scene({
	nodes: 
})

const window = new Window({
	title: "Fps shooter",
	ui: camera,
	onKeyboardEvent: (event: KeyboardEvent) => {
		switch (event.keyCode) {

		}
	}
})