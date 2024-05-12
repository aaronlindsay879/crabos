import sys

files = {
    "test": "this is a text file saved in my initrd file :3",
    "silly": "blehhh :p",
    "big": """
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse risus tellus, ullamcorper at vestibulum sit amet, iaculis sit amet magna. Nam varius metus libero, sit amet tempus eros ullamcorper quis. Vestibulum eget elementum nisl, volutpat sodales neque. Aenean scelerisque convallis ligula, ac ultrices nulla dictum eu. Quisque at libero nibh. Etiam leo purus, interdum non arcu efficitur, faucibus bibendum dui. Nullam varius sed nisl nec feugiat. Nam aliquam rhoncus sapien nec tempus. Nullam ultrices tempor enim eget imperdiet. Nullam dui risus, eleifend vitae mollis ac, gravida quis nunc. Phasellus lobortis gravida maximus. Nam maximus ante et lorem suscipit fringilla. Donec pharetra finibus ornare. Etiam ornare fringilla tristique.

Aliquam imperdiet augue a tortor ullamcorper, et eleifend augue sagittis. Vivamus convallis urna at mi eleifend posuere. Aliquam rutrum aliquet elit, vel aliquam lectus mattis semper. Fusce vehicula vel lacus at blandit. Etiam eget ultrices sem. Aliquam erat volutpat. Sed euismod erat sed magna volutpat tincidunt. Nam rhoncus aliquam congue. In nec ex leo.

Mauris velit ante, lobortis vel hendrerit quis, faucibus non nisl. Duis ut pharetra lacus, sit amet elementum magna. Morbi tristique enim sit amet lacus aliquam, vel rutrum sapien finibus. In sit amet auctor diam. Aenean id erat nisl. Pellentesque et turpis ut orci gravida cursus at eu est. Ut posuere congue dui. Maecenas a pharetra tellus. Praesent in erat hendrerit, finibus risus id, pretium mauris. Donec congue vehicula augue, id blandit nisl pulvinar vel. Donec finibus nulla ac quam dignissim, id pharetra elit fermentum. Pellentesque fringilla nibh vel magna facilisis, sit amet semper urna auctor. Nullam vel enim at lectus venenatis lobortis scelerisque id velit. Pellentesque laoreet tincidunt ante, non rutrum nulla facilisis rutrum. Vestibulum nulla justo, fringilla vitae purus vel, viverra finibus tortor.

Suspendisse ut lacinia sapien, ut porttitor ante. Pellentesque ac pellentesque metus. Etiam in velit tempus, pharetra magna id, vulputate diam. Donec vel enim purus. Pellentesque viverra dictum diam et commodo. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Donec in pretium velit, quis sodales tellus. Donec sagittis, urna at sollicitudin lobortis, est turpis condimentum nunc, sed euismod lacus urna eu massa. Praesent luctus ac velit vel tristique. Vivamus commodo ligula ut ante sagittis, non commodo odio fringilla.

Ut finibus, urna quis sollicitudin vehicula, dolor mi fermentum lorem, sed malesuada tellus nulla ut nisl. Etiam rutrum consectetur justo, vel fermentum elit. Nullam id ante ultricies, iaculis massa vitae, pellentesque nisl. Quisque quis arcu a turpis tincidunt sollicitudin a in lectus. Donec nec eros a tortor dignissim rhoncus. Suspendisse vehicula dictum metus, nec maximus odio vulputate nec. In vitae porttitor nulla, ac ultricies orci. Maecenas porta leo arcu, vel sodales purus accumsan non. Proin volutpat, leo ultricies suscipit vestibulum, nibh quam hendrerit libero, ac consectetur nisl magna vel odio. Sed dui est, interdum scelerisque lacinia in, accumsan et sem. Vestibulum turpis quam, pulvinar quis nisl sed, viverra vestibulum massa. Nunc consectetur consectetur mauris sollicitudin euismod. Quisque fermentum tortor odio, quis dignissim libero tempor quis.

Suspendisse pretium nunc eget odio laoreet, aliquet malesuada turpis scelerisque. Aliquam nec semper justo. Pellentesque orci leo, viverra a nisi in, feugiat tincidunt nunc. Curabitur et rutrum turpis. Aenean lectus diam, mollis interdum pharetra feugiat, sollicitudin vel orci. Pellentesque non magna ante. Praesent et tortor purus. Vivamus eu massa lacus. Aliquam dapibus posuere pellentesque. Vestibulum est risus, fringilla in aliquam nec, interdum ut nisl. Vestibulum justo neque, porttitor pharetra feugiat fringilla, cursus nec dolor. Praesent dignissim gravida lacus, eu sollicitudin ipsum condimentum a. Integer ut vestibulum eros. Maecenas vestibulum dignissim nulla quis semper.

Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Vestibulum nunc nibh, fringilla in accumsan vitae, tincidunt nec nibh. Duis quis laoreet erat, ac facilisis orci. Donec augue purus, vehicula in commodo et, lacinia at leo. Curabitur ac ipsum cursus, sagittis lectus sit amet, rutrum nunc. Phasellus congue porttitor lacinia. Etiam posuere, turpis at mattis placerat, urna odio finibus lectus, ac pulvinar nisi nunc lobortis turpis. Mauris at eros viverra enim lacinia iaculis nec in elit. Donec bibendum est leo. Ut sollicitudin id orci at sollicitudin. Suspendisse eget varius nibh. Ut eget ultricies dolor. Aliquam ante dolor, varius at aliquam ac, placerat in urna.

Donec condimentum blandit quam id luctus. Proin sit amet tristique mauris, eget dignissim purus. Nullam nec suscipit odio. Nam porttitor blandit efficitur. Cras ac ex eros. Donec congue semper nibh, ut bibendum nunc mattis ac. Quisque in dapibus orci. Vivamus facilisis id magna ut bibendum.

Integer iaculis nisl sit amet nisi vulputate laoreet. Curabitur pretium laoreet quam, ac blandit augue. Pellentesque non vehicula risus. Aliquam arcu turpis, malesuada sed massa ut, vehicula vehicula urna. Nullam ut arcu nisl. Cras laoreet cursus laoreet. Sed iaculis tellus ac tristique mollis. Quisque fringilla, augue nec fringilla scelerisque, orci orci fermentum nulla, eu convallis odio enim nec sapien. Praesent consectetur quam ut justo feugiat tincidunt.

Vivamus dictum suscipit risus. Pellentesque varius, arcu vel convallis interdum, mi augue sodales felis, sed accumsan urna odio eget justo. Donec posuere, nunc a laoreet rhoncus, nulla sapien suscipit turpis, nec porttitor odio ipsum a risus. Sed tempus tortor nulla, a sollicitudin mauris venenatis convallis. Fusce sit amet lacus tempus, fermentum lectus non, hendrerit mauris. Morbi iaculis aliquet enim nec congue. Aliquam placerat enim enim, id varius sem rutrum sit amet. Cras vel tortor at sem lacinia porta ut nec metus. Phasellus eget venenatis sapien. Vestibulum rhoncus vitae tellus nec varius. Suspendisse potenti. Aenean laoreet velit at eros rutrum mollis. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Curabitur et pharetra metus. Sed scelerisque accumsan nunc ut viverra.

Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Nulla ultrices sapien sit amet lorem finibus, ac viverra est laoreet. Duis in bibendum nulla. Vivamus non magna tellus. Donec facilisis eros in quam gravida, at aliquet lacus porttitor. Aliquam a euismod mi. Aenean tincidunt neque volutpat fermentum rutrum. Praesent dui elit, pulvinar non odio eu, tincidunt rhoncus massa. Cras vitae velit id nunc sollicitudin vehicula.

Morbi consectetur, est quis pulvinar tincidunt, dolor lectus aliquam dolor, vitae porttitor diam ex eu metus. Suspendisse convallis est nisl, id dignissim purus elementum id. Donec lobortis ornare tortor, eget fermentum lacus dapibus sed. Integer et lorem non metus semper venenatis at a ante. Nunc eget posuere tellus. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi eget mi nec ligula condimentum molestie. Pellentesque luctus euismod nibh in malesuada. Mauris id suscipit metus. In sit amet rutrum urna. Proin elementum accumsan sagittis. Nunc vel erat est. Donec sollicitudin auctor mattis. Morbi in orci id tortor consectetur tempus.

Suspendisse quis mi ut orci consequat ornare a non leo. Ut pretium dignissim velit, in accumsan mauris. Curabitur et tellus sollicitudin, porttitor ipsum eu, dictum enim. Integer erat massa, auctor non orci a, efficitur pharetra arcu. Aenean viverra diam ac lobortis venenatis. Cras a ipsum sagittis enim volutpat varius ut vitae quam. Maecenas dapibus nec leo ut scelerisque. Mauris dapibus luctus magna vel blandit. Vivamus eu orci nec sem feugiat aliquam vel sit amet est. Morbi congue, diam non lacinia blandit, ipsum nibh dignissim risus, quis cursus erat nisi sed augue. Nam commodo purus ut felis condimentum tincidunt ut et nibh. Vivamus commodo venenatis libero, sed aliquam risus eleifend vestibulum. Morbi vitae egestas. """
}
def write_header(file):
    file.write(b"KTIY\0\0\0\0")
    file.write(len(files).to_bytes(8, "little"))
    file.write(sum([len(path) + 1 for (path, content) in files.items()]).to_bytes(8, "little"))

path_index = 0
data_index = 0
def write_file_headers(file):
    global path_index, data_index
    # path_index: usize
    # offset: usize
    # len: usize
    for (path, content) in files.items():
        file.write(path_index.to_bytes(8, "little"))
        file.write(data_index.to_bytes(8, "little"))
        file.write(len(content).to_bytes(8, "little"))

        path_index += len(path) + 1
        data_index += len(content)

def write_string_headers(file):
    for (path, content) in files.items():
        file.write(path.encode())
        file.write(b"\0")

def write_files(file):
    for (path, content) in files.items():
        file.write(content.encode())

path = sys.argv[1]
with open(path, "wb") as file:        
    # u32 - magic number "KTIY"
    # u32 - reserved
    # u64 - header count
    # u64 - string table len
    # headers
    # string table
    # data
    write_header(file)
    write_file_headers(file)
    write_string_headers(file)
    write_files(file)