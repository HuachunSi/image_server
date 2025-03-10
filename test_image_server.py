import requests
import hashlib

# 服务器地址
BASE_URL = "http://image-server.com"  # 图片服务器 endpoint

def calculate_file_hash(file_path):
    """计算文件的 SHA-256 哈希值"""
    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()


def upload_image(file_path):
    """上传图片并返回图片 ID"""
    upload_url = f"{BASE_URL}/upload"
    
    with open(file_path, "rb") as f:
        files = {"file": ("test_image.png", f, "image/png")}
        response = requests.post(upload_url, files=files)
    
    if response.status_code != 200:
        print(f"Upload failed with status code: {response.status_code}")
        print(f"Response: {response.text}")
        return
    
    image_id = response.json()
    return image_id

def download_image(image_id):
    """通过图片 ID 下载图片"""
    download_url = f"{BASE_URL}/download/{image_id}"
    response = requests.get(download_url)
    
    if response.status_code != 200:
        print(f"Download failed with status code: {response.status_code}")
        print(f"Response: {response.text}")
        return
    
    return response.content

def test_upload_and_download():
    # 上传图片
    file_path = "test_image.png"  # 确保这个文件存在
    
    image_id = upload_image(file_path)
    print("Image uploaded successfully")

    content = download_image(image_id)
    # 保存下载的图片
    downloaded_file_path = "/tmp/downloaded_image.png"
    with open(downloaded_file_path, "wb") as f:
        f.write(content)
    print("Image downloaded successfully")

    # 验证原图和下载的图片是否相同
    original_hash = calculate_file_hash(file_path)
    downloaded_hash = calculate_file_hash(downloaded_file_path)

    if original_hash == downloaded_hash:
        print("Verification successful: The downloaded image is identical to the original.")
    else:
        print("Verification failed: The downloaded image is different from the original.")

if __name__ == "__main__":
    test_upload_and_download()