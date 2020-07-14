{
  "targets": [
    {
      "target_name": "binding",
      "sources": [ "native/binding.c" ],
      "include_dirs": [ "gobuild" ],
      "libraries": [ "../gobuild/libesbuild.a" ],
      "conditions": [
        ["OS=='mac'", {
          "libraries": [
            "CoreFoundation.framework",
            "Security.framework"
          ]
        }]
      ]
    }
  ]
}
