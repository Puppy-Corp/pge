import { dlopen, FFIType, JSCallback, suffix } from "bun:ffi"

const path = "../target/debug/libpge.dylib"

const {
	symbols: {
		pge_window_create
	}
} = dlopen(
	path,
	{
		pge_window_create: {
			args: [],
			returns: FFIType.u32
		}
	}
)

const eventCallback = new JSCallback(
	(event) => {},
	{
		returns: FFIType.void,
		args: ["usize"]
	}
)

const res = pge_window_create()

export class Vec3 {
	public x: number
	public y: number
	public z: number
}

enum EulerRot {
    XYZ,
    YXZ,
    ZXY,
    ZYX,
    YZX,
    XZY
}

export class Quat {
	public rotateEuler(rot: EulerRot, a: number, b: number, c: number) {
		
	}
}

class List<T> {

}

export class Node {
	private id: number
	public rotation: Quat
	public translation: Vec3
	public scale: Vec3

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

export type KeyboardEvent = {
	keyCode: KeyCode
	pressed: boolean
}

export type MouseMovedEvent = {
	dx: number
	dy: number
}

export class Window {
	public title?: string
	public ui: UI
	public show: boolean = false

	public constructor(id: number) {
		/*this.title = props.title
		this.ui = props.ui
		if (props.show) {
			this.show = props.show
		}*/
	}

	static async create(props: {
		title?: string
		ui?: UI
		show?: boolean
		onKeyboardEvent?: (event: KeyboardEvent) => void
		onMouseMoved?: (event: MouseMovedEvent) => void
	}): Promise<Window> {
		return new Window(1)
	}
}

export class Material {
	public name?: string
	public normalTexture?: Texture
	public occlusionTexture?: Texture
	public emissiveTexture?: Texture
	public emissiveFactor?: Vec3
}

export class PointLight {
	public color?: Vec3
	public intensity?: number
}

export class Texture {

}

export class Raycast {
	public len?: number
	public intersects?: List<Node>
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