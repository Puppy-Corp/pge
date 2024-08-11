export class Vec3 {
	public x: number
	public y: number
	public z: number
}

class List<T> {

}

export class Node {
	translation = [0, 0, 0];

	children: List<Node | Camera | PointLight | Texture> = [];
}

export class Mesh {
	vertices: number[] = [];
	indices: number[] = [];
}

export class Scene {
	public nodes: Node[] = [];

	public constructor(props: {
		nodes: Node[]
	}) {
		this.nodes = props.nodes
	}
}

export class Target {

}

export class Channel {
	public sampler: Sampler
	public target: Target
}

export class Linear {}
export class Stepm {}
export class CubicSpline {}
export type Inteprolation = Linear | Stepm | CubicSpline

export class AnimationOuput {

}

export class Sampler {
	public input: number[]
	public output: AnimationOuput
	public interpolation: Inteprolation
}

export class Animation {
	public channes: List<Channel>
	public samplers: List<Sampler>
}

export class AnimationPlayer {
	public animation: Animation
}

export class Model3D {
	public textures: List<Texture>
	public materials: List<Material>
	public meshes: List<Mesh>
	public scenes: List<Scene>
	public animations: List<Animation>
	
}

export class Camera {
	public aspect: number
	public fovy: number
	public znear: number
	public zfar: number
}

export enum KeyCode {
	ArrowUp = 38,
	ArrowDown = 40,
	ArrowLeft = 37,
	ArrowRight = 39,
	W = 87,
	A = 65,
	S = 83,
	D = 68
}

export class KeyboardEvent {
	public keyCode
}

export class MouseMoved {
	public dx: number
	public dy: number
}

export class Window {
	public title?: string
	public ui: UI

	public constructor(props: {
		title?: string
		ui?: UI
		onKeyboardEvent?: (event: KeyboardEvent) => void
		onMouseMoved?: (event: MouseMoved) => void
	}) {
		this.title = props.title
		this.ui = props.ui
	}
}

export class Material {
	public name: string
	public normalTexture: Texture
	public occlusionTexture: Texture
	public emissiveTexture: Texture
	public emissiveFactor: Vec3
}

export class PointLight {
	public color: Vec3
	public intensity: number
}

export class Texture {

}

export class Raycast {
	public len: number
	public intersects: List<Node>
}

export const Row = () => {

}

export const Col = () => {

}

export const Cam = () => {

}

export const list = () => {

}

export type UI = any