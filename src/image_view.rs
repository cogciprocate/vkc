
use std::sync::Arc;
use std::ptr;
use vk;
use vks;
use ::{VkcResult, Swapchain, Device};


#[derive(Debug)]
pub struct Inner {
    handle: vk::VkImageView,
    device: Device,
    swapchain: Option<Swapchain>,
}

#[derive(Debug, Clone)]
pub struct ImageView {
    inner: Arc<Inner>,
}

impl ImageView {
    pub fn new(device: Device, swapchain: Option<Swapchain>, image: vk::VkImage, format: vk::VkFormat)
            -> VkcResult<ImageView>
    {
        let create_info = vk::VkImageViewCreateInfo {
            sType: vk::VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            image: image,
            viewType: vk::VK_IMAGE_VIEW_TYPE_2D,
            format: format,
            components: vk::VkComponentMapping {
                r: vk::VK_COMPONENT_SWIZZLE_IDENTITY,
                g: vk::VK_COMPONENT_SWIZZLE_IDENTITY,
                b: vk::VK_COMPONENT_SWIZZLE_IDENTITY,
                a: vk::VK_COMPONENT_SWIZZLE_IDENTITY,
            },
            subresourceRange: vk::VkImageSubresourceRange {
                aspectMask: vk::VK_IMAGE_ASPECT_COLOR_BIT,
                baseMipLevel: 0,
                levelCount: 1,
                baseArrayLayer: 0,
                layerCount: 1,
            },
        };

        let mut handle = 0;

        unsafe {
            ::check(device.vk().core.vkCreateImageView(device.handle(),
                &create_info, ptr::null(), &mut handle));
        }

        Ok(ImageView {
            inner: Arc::new(Inner {
                handle,
                device,
                swapchain,
            })
        })
    }

    pub fn handle(&self) -> vk::VkImageView {
        self.inner.handle
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().core.vkDestroyImageView(self.device.handle(),
                self.handle, ptr::null());
        }
    }
}


pub fn create_image_views(swapchain: &Swapchain) -> VkcResult<Vec<ImageView>> {
    swapchain.images().iter().map(|&image| {
        ImageView::new(swapchain.device().clone(), Some(swapchain.clone()), image, swapchain.image_format())
    }).collect::<Result<Vec<_>, _>>()
}
