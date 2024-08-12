import * as pge from "./pge";

const camera = new pge.Camera()

class Player {
	public node: pge.Node
	public camera: pge.Camera

	constructor() {
		this.node = new pge.Node()
		this.camera = new pge.Camera()
	}

	public rotate(dx: number, dy: number) {
		this.node.rotation.rotateEuler()
	}
}

const player = new Player()

const scene = new pge.Scene({
	nodes: [player.node]
})

const window = await pge.Window.create({
	title: "Fps shooter",
	ui: camera,
	onKeyboardEvent: (e: pge.KeyboardEvent) => {
		switch (e.keyCode) {
			case pge.KeyCode.W:
				player.node.translation.y += 1
				break
		}
	},
	onMouseMoved: (e: pge.MouseMovedEvent) => {
		
	}
})