// swift-tools-version: 5.5
import PackageDescription

let package = Package(
    name: "ModuleTest",
    products: [
        .executable(name: "ModuleTest", targets: ["ModuleTest"]),
    ],
    dependencies: [
        .package(path: "../FfiModule"),
    ],
    targets: [
        .executableTarget(
            name: "ModuleTest",
            dependencies: ["FfiModule"]),
    ]
)
